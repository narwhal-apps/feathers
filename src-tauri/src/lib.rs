pub mod commands;
pub mod error;
pub mod git_core;
pub mod persistence;
pub mod repo_registry;
pub mod watcher;

use crate::persistence::store::{ConfigStore, FileStore};
use crate::repo_registry::RepoRegistry;
use crate::watcher::WatcherRegistry;
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
            let watchers = Arc::new(WatcherRegistry::new(app.handle().clone()));

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
                        match registry.add(std::path::PathBuf::from(path)) {
                            Ok(id) => {
                                if let Ok(h) = registry.get(&id) {
                                    if let Err(e) = watchers.watch(id, h.path.clone()) {
                                        tracing::warn!(
                                            "failed to watch restored repo {path}: {e:?}"
                                        );
                                    }
                                }
                            }
                            Err(e) => tracing::warn!("failed to restore repo {path}: {e:?}"),
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to load config: {e:?}");
                }
            }

            app.manage(registry);
            app.manage(watchers);
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
            commands::branch::branch_create,
            commands::branch::branch_delete,
            commands::branch::branch_rename,
            commands::commit::commit_log,
            commands::commit::commit_create,
            commands::commit::commit_undo,
            commands::diff::diff_workdir,
            commands::diff::diff_index,
            commands::diff::diff_commit,
            commands::stage::stage_files,
            commands::stage::unstage_files,
            commands::discard::discard_files,
            commands::remote::repo_remote_url,
            commands::remote::repo_fetch,
            commands::remote::repo_push,
            commands::remote::repo_publish,
            commands::remote::repo_pull,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
