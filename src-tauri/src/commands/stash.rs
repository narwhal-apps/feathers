use crate::error::AppError;
use crate::git_core::{self, types::FileChange, types::StashEntry};
use crate::repo_registry::RepoRegistry;
use tauri::State;

#[tauri::command]
pub async fn stash_list(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<Vec<StashEntry>, AppError> {
    let handle = registry.get(&id)?;
    let mut r = handle.repo.lock();
    git_core::stash::list(&mut r)
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
    let mut r = handle.repo.lock();
    let oid = git_core::stash::create(&mut r, message.as_deref(), include_untracked, keep_index)?;
    Ok(oid.to_string())
}

#[tauri::command]
pub async fn stash_apply(
    id: String,
    index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let mut r = handle.repo.lock();
    git_core::stash::apply(&mut r, index)
}

#[tauri::command]
pub async fn stash_pop(
    id: String,
    index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let mut r = handle.repo.lock();
    git_core::stash::pop(&mut r, index)
}

#[tauri::command]
pub async fn stash_drop(
    id: String,
    index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let mut r = handle.repo.lock();
    git_core::stash::drop_at(&mut r, index)
}

#[tauri::command]
pub async fn stash_show_files(
    id: String,
    index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<Vec<FileChange>, AppError> {
    let handle = registry.get(&id)?;
    let mut r = handle.repo.lock();
    git_core::stash::show_files(&mut r, index)
}

#[tauri::command]
pub async fn stash_diff_file(
    id: String,
    index: usize,
    path: String,
    registry: State<'_, RepoRegistry>,
) -> Result<String, AppError> {
    let handle = registry.get(&id)?;
    let mut r = handle.repo.lock();
    git_core::stash::diff_file(&mut r, index, &path)
}
