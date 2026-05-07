use crate::error::AppError;
use notify_debouncer_mini::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE_MS: u64 = 250;
pub const REPO_CHANGED_EVENT: &str = "repo_changed";

/// Per-repo filesystem watcher. When changes settle (debounced), emits a
/// `repo_changed` Tauri event so the FE can invalidate that repo's queries.
///
/// We watch the repo root recursively — this covers both worktree edits and
/// `.git/` mutations (terminal commits, branch switches, fetches). The 250ms
/// debounce collapses the bursts of file events that any single git op
/// generates.
pub struct WatcherRegistry {
    inner: Mutex<HashMap<String, Debouncer<RecommendedWatcher>>>,
    app: AppHandle,
}

impl WatcherRegistry {
    pub fn new(app: AppHandle) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            app,
        }
    }

    pub fn watch(&self, id: String, path: PathBuf) -> Result<(), AppError> {
        // Already watching? Replace silently.
        self.unwatch(&id);

        let app = self.app.clone();
        let id_for_thread = id.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(DEBOUNCE_MS),
            move |res: DebounceEventResult| match res {
                Ok(_events) => {
                    if let Err(e) = app.emit(
                        REPO_CHANGED_EVENT,
                        serde_json::json!({ "id": id_for_thread.clone() }),
                    ) {
                        tracing::warn!("failed to emit {REPO_CHANGED_EVENT}: {e}");
                    }
                }
                Err(e) => {
                    tracing::warn!("watcher error for {id_for_thread}: {e:?}");
                }
            },
        )
        .map_err(|e| AppError::Io {
            message: format!("failed to start watcher: {e}"),
        })?;

        debouncer
            .watcher()
            .watch(&path, RecursiveMode::Recursive)
            .map_err(|e| AppError::Io {
                message: format!("failed to watch {}: {e}", path.display()),
            })?;

        self.inner.lock().insert(id, debouncer);
        Ok(())
    }

    pub fn unwatch(&self, id: &str) {
        // Dropping the debouncer terminates the worker thread.
        self.inner.lock().remove(id);
    }
}
