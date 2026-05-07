use crate::error::AppError;
use crate::git_core::{self, types::BranchInfo};
use crate::repo_registry::RepoRegistry;
use tauri::State;

#[tauri::command]
pub async fn branch_list(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<Vec<BranchInfo>, AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::branch::list_branches(&r)
}

#[tauri::command]
pub async fn branch_checkout(
    id: String,
    name: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::branch::checkout(&r, &name)
}

#[tauri::command]
pub async fn branch_create(
    id: String,
    name: String,
    from: Option<String>,
    checkout: bool,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::branch::create(&r, &name, from.as_deref(), checkout)
}

#[tauri::command]
pub async fn branch_delete(
    id: String,
    name: String,
    force: bool,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::branch::delete(&r, &name, force)
}

#[tauri::command]
pub async fn branch_rename(
    id: String,
    old_name: String,
    new_name: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::branch::rename(&r, &old_name, &new_name)
}
