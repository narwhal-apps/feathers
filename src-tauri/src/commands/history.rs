use crate::error::AppError;
use crate::git_core::{self, history::ResetMode};
use crate::repo_registry::RepoRegistry;
use git2::Oid;
use tauri::State;

fn parse_oid(oid: &str) -> Result<Oid, AppError> {
    Oid::from_str(oid).map_err(|e| AppError::Git { message: format!("bad oid: {e}") })
}

#[tauri::command]
pub async fn branch_create_at(
    id: String,
    name: String,
    oid: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::branch::create_at(&r, &name, parse_oid(&oid)?)
}

#[tauri::command]
pub async fn commit_cherrypick(
    id: String,
    oid: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::history::cherrypick(&r, parse_oid(&oid)?)
}

#[tauri::command]
pub async fn commit_revert(
    id: String,
    oid: String,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::history::revert(&r, parse_oid(&oid)?)
}

#[tauri::command]
pub async fn commit_reset(
    id: String,
    oid: String,
    mode: ResetMode,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::history::reset(&r, parse_oid(&oid)?, mode)
}
