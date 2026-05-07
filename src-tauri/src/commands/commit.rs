use crate::error::AppError;
use crate::git_core::{
    self,
    commit::CommitOpts,
    types::{CommitPage, LogOpts},
};
use crate::repo_registry::RepoRegistry;
use tauri::State;

#[tauri::command]
pub async fn commit_log(
    id: String,
    opts: Option<LogOpts>,
    registry: State<'_, RepoRegistry>,
) -> Result<CommitPage, AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::commit::log(&r, opts.unwrap_or_default())
}

#[tauri::command]
pub async fn commit_create(
    id: String,
    message: String,
    opts: Option<CommitOpts>,
    registry: State<'_, RepoRegistry>,
) -> Result<String, AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::commit::create(&r, &message, opts.unwrap_or_default())
}

#[tauri::command]
pub async fn commit_undo(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::commit::undo_last(&r)
}
