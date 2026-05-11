use crate::error::AppError;
use crate::git_core;
use crate::repo_registry::{self, RepoRegistry};
use tauri::State;

#[tauri::command]
pub async fn discard_files(
    id: String,
    paths: Vec<String>,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::discard::discard_paths(r, &paths))
        .await
}

#[tauri::command]
pub async fn discard_hunk(
    id: String,
    path: String,
    hunk_index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| {
        git_core::discard::discard_hunk(r, &path, hunk_index)
    })
    .await
}
