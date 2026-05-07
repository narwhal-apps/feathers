use crate::error::AppError;
use crate::git_core;
use crate::repo_registry::RepoRegistry;
use tauri::State;

#[tauri::command]
pub async fn discard_files(
    id: String,
    paths: Vec<String>,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::discard::discard_paths(&r, &paths)
}

#[tauri::command]
pub async fn discard_hunk(
    id: String,
    path: String,
    hunk_index: usize,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::discard::discard_hunk(&r, &path, hunk_index)
}
