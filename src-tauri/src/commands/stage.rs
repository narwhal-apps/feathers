use crate::error::AppError;
use crate::git_core;
use crate::repo_registry::{self, RepoRegistry};
use tauri::State;

#[tauri::command]
pub async fn stage_files(
    id: String,
    paths: Vec<String>,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::stage::stage_files(r, &paths)).await
}

#[tauri::command]
pub async fn unstage_files(
    id: String,
    paths: Vec<String>,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::stage::unstage_files(r, &paths))
        .await
}
