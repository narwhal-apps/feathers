use crate::error::AppError;
use crate::git_core::{
    self,
    commit::CommitOpts,
    types::{CommitPage, LogOpts},
};
use crate::repo_registry::{self, RepoRegistry};
use tauri::State;

#[tauri::command]
pub async fn commit_log(
    id: String,
    opts: Option<LogOpts>,
    registry: State<'_, RepoRegistry>,
) -> Result<CommitPage, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read(handle, move |r| {
        git_core::commit::log(r, opts.unwrap_or_default())
    })
    .await
}

#[tauri::command]
pub async fn commit_log_unpushed(
    id: String,
    max: Option<usize>,
    registry: State<'_, RepoRegistry>,
) -> Result<CommitPage, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read(handle, move |r| {
        git_core::commit::log_unpushed(r, max.unwrap_or(50))
    })
    .await
}

#[tauri::command]
pub async fn commit_create(
    id: String,
    message: String,
    opts: Option<CommitOpts>,
    registry: State<'_, RepoRegistry>,
) -> Result<String, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| {
        git_core::commit::create(r, &message, opts.unwrap_or_default())
    })
    .await
}

#[tauri::command]
pub async fn commit_undo(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, |r| git_core::commit::undo_last(r)).await
}
