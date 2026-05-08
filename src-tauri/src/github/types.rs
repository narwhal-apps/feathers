use serde::{Deserialize, Serialize};

/// Response from GitHub's `POST /login/device/code`. Sent to the FE so it
/// can show the user the code + verification URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Minimal view of an authenticated user — enough for the titlebar avatar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
}

/// Pull request summary for the PR list. Mirrors GitHub's REST shape, but
/// only the fields we render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub draft: bool,
    pub html_url: String,
    pub user: PrUser,
    pub head: PrRef,
    pub base: PrRef,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrUser {
    pub login: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrRef {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub sha: String,
}

/// Body for `POST /repos/:owner/:repo/pulls`. `body` is optional — leaving
/// the description empty is a valid PR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePullRequestBody {
    pub title: String,
    pub head: String,
    pub base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub draft: bool,
}
