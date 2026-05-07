use crate::error::AppError;
use crate::git_core::types::StatusSnapshot;
use crate::git_core::{self, repo as gc_repo};
use crate::persistence::store::{AppConfig, ConfigStore};
use crate::repo_registry::{RepoId, RepoRegistry, RepoSummary};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct RepoOpenResult {
    pub id: RepoId,
    pub summary: RepoSummary,
    pub head: crate::git_core::types::HeadInfo,
}

fn persist(registry: &RepoRegistry, store: &dyn ConfigStore) {
    let cfg = AppConfig {
        schema: AppConfig::current_schema(),
        known_repos: registry.list().into_iter().map(|s| s.path).collect(),
    };
    if let Err(e) = store.save(&cfg) {
        tracing::warn!("failed to persist known_repos: {e:?}");
    }
}

#[tauri::command]
pub async fn repo_open(
    path: String,
    registry: State<'_, RepoRegistry>,
    store: State<'_, Arc<dyn ConfigStore>>,
) -> Result<RepoOpenResult, AppError> {
    let id = registry.add(PathBuf::from(path))?;
    let handle = registry.get(&id)?;
    let head = {
        let r = handle.repo.lock();
        gc_repo::head_info(&r)?
    };
    let summary = registry
        .list()
        .into_iter()
        .find(|s| s.id == id)
        .ok_or(AppError::RepoNotFound { id: id.clone() })?;
    persist(&registry, store.inner().as_ref());
    Ok(RepoOpenResult { id, summary, head })
}

#[tauri::command]
pub async fn repo_close(
    id: String,
    registry: State<'_, RepoRegistry>,
    store: State<'_, Arc<dyn ConfigStore>>,
) -> Result<(), AppError> {
    registry.remove(&id)?;
    persist(&registry, store.inner().as_ref());
    Ok(())
}

#[tauri::command]
pub async fn repo_list_known(registry: State<'_, RepoRegistry>) -> Result<Vec<RepoSummary>, AppError> {
    Ok(registry.list())
}

#[tauri::command]
pub async fn repo_status(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<StatusSnapshot, AppError> {
    let handle = registry.get(&id)?;
    let r = handle.repo.lock();
    git_core::status::status(&r)
}
