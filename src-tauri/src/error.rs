use serde::Serialize;

#[allow(dead_code)] // variants used by later milestones (M2+)
#[derive(thiserror::Error, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppError {
    #[error("repository not found: {id}")]
    RepoNotFound { id: String },

    #[error("working tree dirty: {} path(s)", paths.len())]
    Dirty { paths: Vec<String> },

    #[error("merge conflict in {} path(s)", paths.len())]
    MergeConflict { paths: Vec<String> },

    #[error("authentication failed: {message}")]
    Auth { message: String },

    #[error("github rate-limited; retry after {retry_after}s")]
    GithubRateLimited { retry_after: u64 },

    #[error("not a github repository")]
    NotAGithubRepo,

    #[error("network error: {message}")]
    Network { message: String },

    #[error("io error: {message}")]
    Io { message: String },

    #[error("git error: {message}")]
    Git { message: String },

    #[error("operation cancelled")]
    Cancelled,
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            message: e.to_string(),
        }
    }
}

impl From<git2::Error> for AppError {
    fn from(e: git2::Error) -> Self {
        Self::Git { message: e.message().to_string() }
    }
}
