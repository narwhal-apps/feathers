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
