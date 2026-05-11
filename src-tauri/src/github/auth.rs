use crate::error::AppError;
use crate::github::types::DeviceCodeResponse;
#[cfg(not(debug_assertions))]
use keyring::Entry;
use parking_lot::Mutex;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[cfg(not(debug_assertions))]
const KEYRING_SERVICE: &str = "feathers-github";
#[cfg(not(debug_assertions))]
const KEYRING_ACCOUNT: &str = "default";

/// In debug builds we store the OAuth token in a plaintext file under
/// `app_data_dir/dev-token` instead of the macOS Keychain. Reason: dev
/// binaries are unsigned (or re-signed every rebuild), so the Keychain ACL
/// doesn't recognise them and macOS prompts for the login password on every
/// launch. The file backend trades that prompt for a plaintext token in the
/// developer's own profile dir — acceptable since dev tokens are scoped to
/// the developer's personal OAuth app.
///
/// Initialised once at startup by `lib.rs` via `init_dev_token_path`. In
/// release builds the path is never read; the keychain backend is used.
static DEV_TOKEN_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Called from `lib.rs` setup() to give the dev backend a writable path.
/// No-op in release builds.
pub fn init_dev_token_path(path: PathBuf) {
    let _ = DEV_TOKEN_PATH.set(path);
}

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

#[cfg(not(debug_assertions))]
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
    write_backend(token)?;
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
    // Cache miss — first call this session. One backend read.
    let loaded = read_backend()?;
    cache().lock().state = Some(loaded.clone());
    Ok(loaded)
}

pub fn clear_token() -> Result<(), AppError> {
    let result = delete_backend();
    cache().lock().state = Some(None);
    result
}

#[cfg(not(debug_assertions))]
fn write_backend(token: &str) -> Result<(), AppError> {
    entry()?
        .set_password(token)
        .map_err(|e| AppError::Auth { message: e.to_string() })
}

#[cfg(not(debug_assertions))]
fn read_backend() -> Result<Option<String>, AppError> {
    match entry()?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Auth { message: e.to_string() }),
    }
}

#[cfg(not(debug_assertions))]
fn delete_backend() -> Result<(), AppError> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Auth { message: e.to_string() }),
    }
}

#[cfg(debug_assertions)]
fn dev_path() -> Result<&'static PathBuf, AppError> {
    DEV_TOKEN_PATH.get().ok_or_else(|| AppError::Auth {
        message: "dev token path not initialised — call init_dev_token_path() at startup".into(),
    })
}

#[cfg(debug_assertions)]
fn write_backend(token: &str) -> Result<(), AppError> {
    let path = dev_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Auth {
            message: format!("create dev token dir: {e}"),
        })?;
    }
    std::fs::write(path, token).map_err(|e| AppError::Auth {
        message: format!("write dev token: {e}"),
    })
}

#[cfg(debug_assertions)]
fn read_backend() -> Result<Option<String>, AppError> {
    let path = dev_path()?;
    match std::fs::read_to_string(path) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::Auth {
            message: format!("read dev token: {e}"),
        }),
    }
}

#[cfg(debug_assertions)]
fn delete_backend() -> Result<(), AppError> {
    let path = dev_path()?;
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Auth {
            message: format!("delete dev token: {e}"),
        }),
    }
}
