use crate::error::AppError;
use crate::git_core::remote;
use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks};
use std::path::Path;

/// Clone a remote repository into `dest`. The destination must not already
/// exist (libgit2 refuses to clone into a non-empty directory).
pub fn clone(url: &str, dest: &Path) -> Result<(), AppError> {
    if dest.exists() {
        return Err(AppError::Io {
            message: format!("destination already exists: {}", dest.display()),
        });
    }

    let mut cbs = RemoteCallbacks::new();
    cbs.credentials(remote::credentials_cb);

    let mut fetch = FetchOptions::new();
    fetch.remote_callbacks(cbs);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch);

    builder
        .clone(url, dest)
        .map(|_| ())
        .map_err(|e| AppError::Git {
            message: e.message().to_string(),
        })
}
