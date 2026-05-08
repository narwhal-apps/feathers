use crate::error::AppError;
use crate::git_core;
use crate::github::api;
use crate::github::auth;
use crate::github::types::{
    CreatePullRequestBody, DeviceCodeResponse, GitHubUser, PullRequest,
};
use crate::repo_registry::RepoRegistry;
use tauri::State;

#[tauri::command]
pub async fn github_start_device_flow() -> Result<DeviceCodeResponse, AppError> {
    auth::start_device_flow().await
}

#[tauri::command]
pub async fn github_complete_device_flow(
    device_code: String,
    interval: u64,
) -> Result<(), AppError> {
    auth::complete_device_flow(&device_code, interval).await
}

#[tauri::command]
pub async fn github_signout() -> Result<(), AppError> {
    auth::clear_token()
}

/// Returns the signed-in GitHub user, or `None` if no token is stored.
/// Returning `None` (not an error) for the no-token case keeps the FE
/// init flow simple — startup never alerts.
#[tauri::command]
pub async fn github_user() -> Result<Option<GitHubUser>, AppError> {
    if auth::load_token()?.is_none() {
        return Ok(None);
    }
    match api::me().await {
        Ok(u) => Ok(Some(u)),
        Err(AppError::Auth { .. }) => {
            // Token rejected — drop it so the UI shows signed-out state.
            let _ = auth::clear_token();
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn github_list_prs(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<Vec<PullRequest>, AppError> {
    let url = {
        let handle = registry.get(&id)?;
        let r = handle.repo.lock();
        git_core::remote::url(&r, None)?
    };
    let url = url.ok_or_else(|| AppError::Git {
        message: "no 'origin' remote configured".into(),
    })?;
    let (owner, repo) = api::parse_github_url(&url).ok_or(AppError::NotAGithubRepo)?;
    api::list_pull_requests(&owner, &repo).await
}

#[tauri::command]
pub async fn github_create_pr(
    id: String,
    title: String,
    body: Option<String>,
    base: String,
    draft: bool,
    registry: State<'_, RepoRegistry>,
) -> Result<PullRequest, AppError> {
    let (url, head_branch) = {
        let handle = registry.get(&id)?;
        let r = handle.repo.lock();
        let url = git_core::remote::url(&r, None)?;
        let head_ref = r.head()?;
        let head_name = head_ref
            .shorthand()
            .ok_or_else(|| AppError::Git {
                message: "HEAD is detached or has no branch".into(),
            })?
            .to_string();
        (url, head_name)
    };
    let url = url.ok_or_else(|| AppError::Git {
        message: "no 'origin' remote configured".into(),
    })?;
    let (owner, repo) = api::parse_github_url(&url).ok_or(AppError::NotAGithubRepo)?;
    if head_branch == base {
        return Err(AppError::Git {
            message: format!("head and base are both '{base}'"),
        });
    }
    let payload = CreatePullRequestBody {
        title,
        head: head_branch,
        base,
        body: body.filter(|s| !s.trim().is_empty()),
        draft,
    };
    api::create_pull_request(&owner, &repo, &payload).await
}
