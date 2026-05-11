use crate::error::AppError;
use crate::github::auth;
use crate::github::types::{CreatePullRequestBody, GitHubUser, PullRequest};
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Serialize;

const API_BASE: &str = "https://api.github.com";

/// Convert a non-2xx GitHub response into the right `AppError` variant.
///
/// GitHub uses 403 for several distinct conditions and they need different UX:
///   - Primary rate limit: `X-RateLimit-Remaining: 0` set on the response
///   - Secondary rate limit (burst): 429 status, or 403 with body
///     `"You have exceeded a secondary rate limit"`
///   - Authorization failure (SAML SSO required, OAuth app not approved by
///     org, missing token scope, repo permissions): plain 403 with a body
///     `message` like "Resource not accessible by personal access token" or
///     "Must have admin rights to Repository"
///
/// Only the first two are real rate limits. Everything else is surfaced as
/// `Network { message: <github's message> }` so the UI shows the real cause.
async fn map_error(resp: Response) -> AppError {
    let status = resp.status();

    if status.as_u16() == 401 {
        return AppError::Auth {
            message: "GitHub token rejected — sign in again".into(),
        };
    }

    let headers = resp.headers().clone();
    let raw_body = resp.text().await.unwrap_or_default();
    let body_message = serde_json::from_str::<serde_json::Value>(&raw_body)
        .ok()
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(str::to_string));

    let is_secondary_rate_limit = body_message
        .as_deref()
        .map(|m| m.to_lowercase().contains("secondary rate limit"))
        .unwrap_or(false);
    let is_primary_rate_limit = headers
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .map(|s| s == "0")
        .unwrap_or(false);

    if status.as_u16() == 429 || is_primary_rate_limit || is_secondary_rate_limit {
        let retry_after = headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| {
                // Fall back to X-RateLimit-Reset (unix epoch when the window
                // resets) if Retry-After isn't present.
                let reset = headers
                    .get("x-ratelimit-reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())?;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();
                reset.checked_sub(now)
            })
            .unwrap_or(60);
        return AppError::GithubRateLimited { retry_after };
    }

    let msg = body_message.unwrap_or_else(|| {
        if raw_body.is_empty() {
            format!("github {status}")
        } else {
            format!("github {status}: {raw_body}")
        }
    });
    AppError::Network { message: msg }
}

async fn get<T: DeserializeOwned>(url: &str) -> Result<T, AppError> {
    let token = auth::load_token()?.ok_or_else(|| AppError::Auth {
        message: "not signed in to GitHub".into(),
    })?;
    let resp = crate::github::client()
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })?;

    if !resp.status().is_success() {
        return Err(map_error(resp).await);
    }
    resp.json::<T>()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })
}

async fn post<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, AppError> {
    let token = auth::load_token()?.ok_or_else(|| AppError::Auth {
        message: "not signed in to GitHub".into(),
    })?;
    let resp = crate::github::client()
        .post(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", format!("Bearer {token}"))
        .json(body)
        .send()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })?;

    if !resp.status().is_success() {
        return Err(map_error(resp).await);
    }
    resp.json::<T>()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })
}

pub async fn me() -> Result<GitHubUser, AppError> {
    get(&format!("{API_BASE}/user")).await
}

pub async fn list_pull_requests(owner: &str, repo: &str) -> Result<Vec<PullRequest>, AppError> {
    let url = format!(
        "{API_BASE}/repos/{owner}/{repo}/pulls?state=open&sort=updated&direction=desc&per_page=50"
    );
    get(&url).await
}

pub async fn create_pull_request(
    owner: &str,
    repo: &str,
    body: &CreatePullRequestBody,
) -> Result<PullRequest, AppError> {
    let url = format!("{API_BASE}/repos/{owner}/{repo}/pulls");
    post(&url, body).await
}

/// Pull (owner, repo) out of a Git remote URL when it points at github.com.
/// Returns None for non-GitHub remotes or unrecognized URL shapes.
pub fn parse_github_url(url: &str) -> Option<(String, String)> {
    let strip = |s: &str| s.trim_end_matches(".git").trim_end_matches('/').to_string();
    let split = |rest: String| {
        let mut parts = rest.splitn(2, '/');
        let owner = parts.next()?.to_string();
        let repo = parts.next()?.to_string();
        if owner.is_empty() || repo.is_empty() {
            None
        } else {
            Some((owner, repo))
        }
    };
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        return split(strip(rest));
    }
    if let Some(rest) = url.strip_prefix("ssh://git@github.com/") {
        return split(strip(rest));
    }
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        return split(strip(rest));
    }
    if let Some(rest) = url.strip_prefix("http://github.com/") {
        return split(strip(rest));
    }
    if let Some(rest) = url.strip_prefix("git://github.com/") {
        return split(strip(rest));
    }
    None
}
