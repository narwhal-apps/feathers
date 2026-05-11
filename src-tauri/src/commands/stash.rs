use crate::error::AppError;
use crate::git_core::{self, types::DiffPayload, types::FileChange, types::StashEntry};
use crate::repo_registry::{self, RepoRegistry};
use tauri::State;

#[tauri::command]
pub async fn stash_list(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<Vec<StashEntry>, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read_mut(handle, git_core::stash::list).await
}

#[tauri::command]
pub async fn stash_create(
    id: String,
    message: Option<String>,
    include_untracked: bool,
    keep_index: bool,
    registry: State<'_, RepoRegistry>,
) -> Result<String, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| {
        let oid = git_core::stash::create(r, message.as_deref(), include_untracked, keep_index)?;
        Ok(oid.to_string())
    })
    .await
}

#[tauri::command]
pub async fn stash_apply(
    id: String,
    index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::stash::apply(r, index)).await
}

#[tauri::command]
pub async fn stash_pop(
    id: String,
    index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::stash::pop(r, index)).await
}

#[tauri::command]
pub async fn stash_drop(
    id: String,
    index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::stash::drop_at(r, index)).await
}

#[tauri::command]
pub async fn stash_show_files(
    id: String,
    index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<Vec<FileChange>, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read_mut(handle, move |r| git_core::stash::show_files(r, index)).await
}

#[tauri::command]
pub async fn stash_diff_file(
    id: String,
    index: usize,
    path: String,
    registry: State<'_, RepoRegistry>,
) -> Result<DiffPayload, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read_mut(handle, move |r| git_core::stash::diff_file(r, index, &path))
        .await
}
