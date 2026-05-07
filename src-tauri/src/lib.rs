pub mod commands;
pub mod error;
pub mod git_core;
pub mod persistence;
pub mod repo_registry;

use crate::persistence::store::{ConfigStore, FileStore};
use crate::repo_registry::RepoRegistry;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init()
        .ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let registry = RepoRegistry::new();

            // Restore known repos from our JSON sidecar.
            let cfg_path = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("app_data_dir: {e}"))?
                .join("config.json");
            let store: Arc<dyn ConfigStore> = Arc::new(FileStore::new(cfg_path.clone()));

            match store.load() {
                Ok(cfg) => {
                    for path in &cfg.known_repos {
                        if let Err(e) = registry.add(std::path::PathBuf::from(path)) {
                            tracing::warn!("failed to restore repo {path}: {e:?}");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to load config: {e:?}");
                }
            }

            app.manage(registry);
            app.manage(store);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_version,
            commands::repo::repo_open,
            commands::repo::repo_close,
            commands::repo::repo_list_known,
            commands::repo::repo_status,
            commands::branch::branch_list,
            commands::branch::branch_checkout,
            commands::commit::commit_log,
            commands::commit::commit_create,
            commands::diff::diff_workdir,
            commands::diff::diff_index,
            commands::diff::diff_commit,
            commands::stage::stage_files,
            commands::stage::unstage_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
