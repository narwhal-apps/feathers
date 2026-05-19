use crate::error::AppError;
use crate::git_core::{self, types::BranchInfo};
use crate::repo_registry::{self, RepoRegistry};
use tauri::State;

#[tauri::command]
pub async fn branch_list(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<Vec<BranchInfo>, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read(handle, git_core::branch::list_branches).await
}

#[tauri::command]
pub async fn branch_checkout(
    id: String,
    name: String,
    allow_dirty: Option<bool>,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let ad = allow_dirty.unwrap_or(false);
    repo_registry::with_repo_write(handle, move |r| {
        git_core::branch::checkout(r, &name, ad)
    })
    .await
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
    repo_registry::with_repo_write(handle, move |r| {
        git_core::branch::create(r, &name, from.as_deref(), checkout)
    })
    .await
}

#[tauri::command]
pub async fn branch_delete(
    id: String,
    name: String,
    force: bool,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::branch::delete(r, &name, force))
        .await
}

#[tauri::command]
pub async fn branch_rename(
    id: String,
    old_name: String,
    new_name: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| {
        git_core::branch::rename(r, &old_name, &new_name)
    })
    .await
}
