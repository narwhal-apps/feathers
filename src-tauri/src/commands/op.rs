use crate::error::AppError;
use crate::git_core::{self, op::OpState};
use crate::repo_registry::RepoRegistry;
use tauri::State;

#[tauri::command]
pub async fn repo_op_state(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<OpState, AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::op::state(&r)
}

#[tauri::command]
pub async fn repo_op_continue(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::op::op_continue(&r)
}

#[tauri::command]
pub async fn repo_op_abort(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::op::op_abort(&r)
}
