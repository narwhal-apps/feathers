use crate::error::AppError;
use crate::git_core::{self, types::DiffPayload};
use crate::repo_registry::{self, RepoRegistry};
use tauri::State;

#[tauri::command]
pub async fn diff_workdir(
    id: String,
    paths: Option<Vec<String>>,
    registry: State<'_, RepoRegistry>,
) -> Result<DiffPayload, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read(handle, move |r| git_core::diff::diff_workdir(r, paths)).await
}

#[tauri::command]
pub async fn diff_index(
    id: String,
    paths: Option<Vec<String>>,
    registry: State<'_, RepoRegistry>,
) -> Result<DiffPayload, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read(handle, move |r| git_core::diff::diff_index(r, paths)).await
}

#[tauri::command]
pub async fn diff_commit(
    id: String,
    oid: String,
    registry: State<'_, RepoRegistry>,
) -> Result<DiffPayload, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read(handle, move |r| git_core::diff::diff_commit(r, &oid)).await
}
