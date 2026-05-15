pub mod commands;
pub mod error;
pub mod git_core;
pub mod github;
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
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let registry = RepoRegistry::new();
            let watchers = Arc::new(WatcherRegistry::new(app.handle().clone()));

            let app_data = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("app_data_dir: {e}"))?;

            // Dev builds store the GitHub OAuth token in a file under
            // app_data_dir to avoid the macOS Keychain prompt on every
            // launch (unsigned/re-signed dev binaries don't match the ACL).
            // No-op in release builds.
            crate::github::auth::init_dev_token_path(app_data.join("dev-token"));

            // Restore known repos from our JSON sidecar.
            let cfg_path = app_data.join("config.json");
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
            commands::repo::repo_clone,
            commands::repo::repo_close,
            commands::repo::repo_list_known,
            commands::repo::repo_status,
            commands::branch::branch_list,
            commands::branch::branch_checkout,
            commands::branch::branch_create,
            commands::branch::branch_delete,
            commands::branch::branch_rename,
            commands::commit::commit_log,
            commands::commit::commit_log_unpushed,
            commands::commit::commit_create,
            commands::commit::commit_undo,
            commands::diff::diff_workdir,
            commands::diff::diff_index,
            commands::diff::diff_commit,
            commands::stage::stage_files,
            commands::stage::unstage_files,
            commands::discard::discard_files,
            commands::discard::discard_hunk,
            commands::op::repo_op_state,
            commands::op::repo_op_continue,
            commands::op::repo_op_abort,
            commands::editor::repo_open_in_editor,
            commands::github::github_start_device_flow,
            commands::github::github_complete_device_flow,
            commands::github::github_signout,
            commands::github::github_user,
            commands::github::github_list_prs,
            commands::github::github_create_pr,
            commands::remote::repo_remote_url,
            commands::remote::repo_fetch,
            commands::remote::repo_push,
            commands::remote::repo_publish,
            commands::remote::repo_pull,
            commands::settings::settings_get,
            commands::settings::settings_set_theme,
            commands::settings::settings_set_last_active_repo_path,
            commands::settings::settings_get_git_identity,
            commands::settings::settings_set_git_identity,
            commands::window::open_settings_window,
            commands::history::branch_create_at,
            commands::history::commit_cherrypick,
            commands::history::commit_revert,
            commands::history::commit_reset,
            commands::stash::stash_list,
            commands::stash::stash_create,
            commands::stash::stash_apply,
            commands::stash::stash_pop,
            commands::stash::stash_drop,
            commands::stash::stash_show_files,
            commands::stash::stash_diff_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
