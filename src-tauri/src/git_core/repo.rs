use crate::error::AppError;
use crate::git_core::types::HeadInfo;
use git2::Repository;
use std::path::Path;

pub fn open(path: &Path) -> Result<Repository, AppError> {
    Repository::open(path).map_err(Into::into)
}

pub fn head_info(repo: &Repository) -> Result<HeadInfo, AppError> {
    match repo.head() {
        Ok(head) => {
            let detached = repo.head_detached().unwrap_or(false);
            let branch = if detached {
                "HEAD".to_string()
            } else {
                head.shorthand().unwrap_or("HEAD").to_string()
            };
            let short_sha = head
                .target()
                .and_then(|oid| repo.find_object(oid, None).ok())
                .and_then(|obj| obj.short_id().ok())
                .and_then(|buf| buf.as_str().map(str::to_string))
                .unwrap_or_default();
            Ok(HeadInfo { branch, detached, short_sha })
        }
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch || e.code() == git2::ErrorCode::NotFound => {
            // Empty repo with no commits.
            Ok(HeadInfo { branch: "HEAD".to_string(), detached: false, short_sha: String::new() })
        }
        Err(e) => Err(e.into()),
    }
}
