use crate::error::AppError;
use crate::github::auth;
use crate::github::types::{CreatePullRequestBody, GitHubUser, PullRequest};
use serde::de::DeserializeOwned;
use serde::Serialize;

const API_BASE: &str = "https://api.github.com";

fn http() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .user_agent("feathers")
        .build()
        .map_err(|e| AppError::Network { message: e.to_string() })
}

async fn get<T: DeserializeOwned>(url: &str) -> Result<T, AppError> {
    let token = auth::load_token()?.ok_or_else(|| AppError::Auth {
        message: "not signed in to GitHub".into(),
    })?;
    let resp = http()?
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })?;

    let status = resp.status();
    if status.as_u16() == 401 {
        return Err(AppError::Auth {
            message: "GitHub token rejected — sign in again".into(),
        });
    }
    if status.as_u16() == 403 || status.as_u16() == 429 {
        // Primary or secondary rate limit.
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);
        return Err(AppError::GithubRateLimited { retry_after });
    }
    if !status.is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(AppError::Network {
            message: format!("github {status}: {msg}"),
        });
    }
    resp.json::<T>()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })
}

async fn post<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, AppError> {
    let token = auth::load_token()?.ok_or_else(|| AppError::Auth {
        message: "not signed in to GitHub".into(),
    })?;
    let resp = http()?
        .post(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", format!("Bearer {token}"))
        .json(body)
        .send()
        .await
        .map_err(|e| AppError::Network { message: e.to_string() })?;
    let status = resp.status();
    if status.as_u16() == 401 {
        return Err(AppError::Auth {
            message: "GitHub token rejected — sign in again".into(),
        });
    }
    if status.as_u16() == 403 || status.as_u16() == 429 {
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);
        return Err(AppError::GithubRateLimited { retry_after });
    }
    if !status.is_success() {
        // Surface GitHub's "message" field when present so the FE shows the
        // real reason ("No commits between main and feat/foo", etc.) instead
        // of an opaque HTTP status.
        let raw = resp.text().await.unwrap_or_default();
        let pretty = serde_json::from_str::<serde_json::Value>(&raw)
            .ok()
            .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(str::to_string))
            .unwrap_or(raw);
        return Err(AppError::Network {
            message: format!("github {status}: {pretty}"),
        });
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
