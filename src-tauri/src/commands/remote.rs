use crate::error::AppError;
use crate::git_core;
use crate::repo_registry::{self, RepoRegistry};
use tauri::State;

#[tauri::command]
pub async fn repo_remote_url(
    id: String,
    remote: Option<String>,
    registry: State<'_, RepoRegistry>,
) -> Result<Option<String>, AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_read(handle, move |r| git_core::remote::url(r, remote.as_deref()))
        .await
}

#[tauri::command]
pub async fn repo_fetch(
    id: String,
    remote: Option<String>,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::remote::fetch(r, remote.as_deref()))
        .await
}

#[tauri::command]
pub async fn repo_push(
    id: String,
    remote: Option<String>,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| git_core::remote::push(r, remote.as_deref()))
        .await
}

#[tauri::command]
pub async fn repo_publish(
    id: String,
    remote: Option<String>,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| {
        git_core::remote::publish(r, remote.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn repo_pull(
    id: String,
    remote: Option<String>,
    rebase: bool,
    registry: State<'_, RepoRegistry>,
) -> Result<(), AppError> {
    let handle = registry.get(&id)?;
    repo_registry::with_repo_write(handle, move |r| {
        git_core::remote::pull(r, remote.as_deref(), rebase)
    })
    .await
}
