use crate::error::AppError;
use crate::github::types::DeviceCodeResponse;
use keyring::Entry;
use parking_lot::Mutex;
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const KEYRING_SERVICE: &str = "feathers-github";
const KEYRING_ACCOUNT: &str = "default";

/// GitHub OAuth client_id. Set at build time via the `GITHUB_CLIENT_ID`
/// env var. Without it sign-in returns a clear error so devs can register
/// their own OAuth app at https://github.com/settings/developers and
/// rebuild with `GITHUB_CLIENT_ID=Iv1.xxx cargo build`.
const CLIENT_ID: Option<&str> = option_env!("GITHUB_CLIENT_ID");
const SCOPES: &str = "repo,read:user";

const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

fn client_id() -> Result<&'static str, AppError> {
    CLIENT_ID.ok_or_else(|| AppError::Auth {
        message: "GITHUB_CLIENT_ID was not set at build time. Register a \
                  GitHub OAuth app (with device flow enabled) and rebuild \
                  with GITHUB_CLIENT_ID=<id> cargo build."
            .into(),
    })
}

fn http() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .user_agent("feathers")
        .build()
        .map_err(|e| AppError::Network { message: e.to_string() })
}

/// Step 1 of the device flow — ask GitHub for a device code + user code.
pub async fn start_device_flow() -> Result<DeviceCodeResponse, AppError> {
    let id = client_id()?;
    let resp = http()?
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&[("client_id", id), ("scope", SCOPES)])
        .send()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })?;
    if !resp.status().is_success() {
        return Err(AppError::Network {
            message: format!("device code request failed: {}", resp.status()),
        });
    }
    resp.json::<DeviceCodeResponse>()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })
}

/// Step 2 — poll until the user authorizes the device, then store the token
/// in the OS keychain. Honors GitHub's `slow_down` and `authorization_pending`
/// signals; aborts on `expired_token`/`access_denied`/timeout.
pub async fn complete_device_flow(device_code: &str, interval_secs: u64) -> Result<(), AppError> {
    let id = client_id()?;
    let client = http()?;
    let start = Instant::now();
    let timeout = Duration::from_secs(900); // 15 min — GitHub device codes expire after ~15
    let mut interval = interval_secs.max(5);

    loop {
        if start.elapsed() > timeout {
            return Err(AppError::Auth {
                message: "device flow timed out — start sign-in again".into(),
            });
        }
        tokio::time::sleep(Duration::from_secs(interval)).await;

        #[derive(Deserialize)]
        struct TokenResponse {
            #[serde(default)]
            access_token: Option<String>,
            #[serde(default)]
            error: Option<String>,
        }

        let resp = client
            .post(TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", id),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .map_err(|e| AppError::Network { message: e.to_string() })?;

        let body: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Network { message: e.to_string() })?;

        if let Some(token) = body.access_token {
            store_token(&token)?;
            return Ok(());
        }
        match body.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                interval += 5;
                continue;
            }
            Some("expired_token") => {
                return Err(AppError::Auth {
                    message: "device code expired — start sign-in again".into(),
                })
            }
            Some("access_denied") => {
                return Err(AppError::Auth {
                    message: "access denied".into(),
                })
            }
            Some(other) => {
                return Err(AppError::Auth {
                    message: format!("oauth error: {other}"),
                })
            }
            None => continue, // unknown response shape — keep polling
        }
    }
}

fn entry() -> Result<Entry, AppError> {
    Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| AppError::Auth { message: e.to_string() })
}

/// Process-local cache keyed by "have we tried loading from the Keychain
/// at least once?". macOS prompts the user for their login password every
/// time an unsigned (or re-signed) binary reads a Keychain item — for dev
/// builds that means a prompt per API call. Caching the token in-process
/// reduces the prompt to AT MOST one per app launch (and zero for the rest
/// of the session after sign-in, since `store_token` populates the cache).
struct Cache {
    /// `Some(value)` once we've consulted the Keychain. The inner Option
    /// distinguishes "loaded, no token stored" from "haven't loaded yet".
    state: Option<Option<String>>,
}

static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
fn cache() -> &'static Mutex<Cache> {
    CACHE.get_or_init(|| Mutex::new(Cache { state: None }))
}

pub fn store_token(token: &str) -> Result<(), AppError> {
    entry()?
        .set_password(token)
        .map_err(|e| AppError::Auth { message: e.to_string() })?;
    cache().lock().state = Some(Some(token.to_string()));
    Ok(())
}

pub fn load_token() -> Result<Option<String>, AppError> {
    {
        let c = cache().lock();
        if let Some(loaded) = c.state.as_ref() {
            return Ok(loaded.clone());
        }
    }
    // Cache miss — first call this session. One Keychain read.
    let loaded = match entry()?.get_password() {
        Ok(t) => Some(t),
        Err(keyring::Error::NoEntry) => None,
        Err(e) => return Err(AppError::Auth { message: e.to_string() }),
    };
    cache().lock().state = Some(loaded.clone());
    Ok(loaded)
}

pub fn clear_token() -> Result<(), AppError> {
    let result = match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Auth { message: e.to_string() }),
    };
    cache().lock().state = Some(None);
    result
}
