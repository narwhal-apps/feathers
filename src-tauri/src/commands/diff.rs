use crate::error::AppError;
use crate::git_core::{self, types::DiffPayload};
use crate::repo_registry::RepoRegistry;
use tauri::State;

#[tauri::command]
pub async fn diff_workdir(
    id: String,
    paths: Option<Vec<String>>,
    registry: State<'_, RepoRegistry>,
) -> Result<DiffPayload, AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::diff::diff_workdir(&r, paths)
}

#[tauri::command]
pub async fn diff_index(
    id: String,
    paths: Option<Vec<String>>,
    registry: State<'_, RepoRegistry>,
) -> Result<DiffPayload, AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::diff::diff_index(&r, paths)
}

#[tauri::command]
pub async fn diff_commit(
    id: String,
    oid: String,
    registry: State<'_, RepoRegistry>,
) -> Result<DiffPayload, AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::diff::diff_commit(&r, &oid)
}
