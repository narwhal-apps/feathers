use crate::error::AppError;
use notify_debouncer_mini::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE_MS: u64 = 600;
pub const REPO_CHANGED_EVENT: &str = "repo_changed";

/// Per-repo filesystem watcher. When changes settle (debounced), emits a
/// `repo_changed` Tauri event so the FE can invalidate that repo's queries.
///
/// We watch the repo root recursively — this covers both worktree edits and
/// `.git/` mutations (terminal commits, branch switches, fetches). The 600ms
/// debounce coalesces the inner bursts of `.git/objects/` writes that fetch
/// and commit produce. Pure-noise batches (only `.git/objects/`, locks, etc.)
/// are dropped without emitting; the remaining batches carry a `kind` hint so
/// the FE can decide whether to invalidate just `status`/`op-state` or also
/// `branches`/`log`.
pub struct WatcherRegistry {
    inner: Mutex<HashMap<String, Debouncer<RecommendedWatcher>>>,
    app: AppHandle,
}

#[derive(serde::Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
enum EventKind {
    Refs,
    Workdir,
}

/// Classify a single FS event path relative to the repo root.
///
/// Returns `None` for noise paths the FE should never see (object writes, lfs
/// blobs, hook installs, `.lock`/`.tmp` scratch files). Otherwise returns
/// `Refs` for paths that signal branch/log/op-state changes, or `Workdir` for
/// everything else (working-tree edits, `.git/index`, `.git/logs/`, …).
fn classify(path: &Path, repo_root: &Path) -> Option<EventKind> {
    // Try to resolve path relative to repo_root for cleaner matching. notify
    // gives absolute paths on macOS; if strip fails (rare — symlinks etc.),
    // fall back to the absolute path and rely on the substring checks below.
    let rel = path.strip_prefix(repo_root).unwrap_or(path);
    let s = rel.to_string_lossy();

    // Drop pure noise. `.git/objects/` is the dominant offender during fetch
    // and gc; `.lock`/`.tmp` are git's atomic-write scratch files.
    if s.contains(".git/objects/")
        || s.contains(".git/lfs/")
        || s.contains(".git/hooks/")
        || s.ends_with(".lock")
        || s.ends_with(".tmp")
    {
        return None;
    }

    // Refs: anything that signals branches/log/op-state changes.
    if s.starts_with(".git/refs/")
        || s == ".git/HEAD"
        || s == ".git/packed-refs"
        || s == ".git/MERGE_HEAD"
        || s == ".git/MERGE_MSG"
        || s == ".git/CHERRY_PICK_HEAD"
        || s == ".git/REVERT_HEAD"
        || s == ".git/FETCH_HEAD"
        || s == ".git/ORIG_HEAD"
        || s.starts_with(".git/feathers/")
    {
        return Some(EventKind::Refs);
    }

    // Anything else (working tree, `.git/index`, `.git/logs/`, …) → workdir.
    Some(EventKind::Workdir)
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
        let repo_root = path.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(DEBOUNCE_MS),
            move |res: DebounceEventResult| match res {
                Ok(events) => {
                    // Reduce the batch to a single kind. If any path classifies
                    // as Refs, the whole batch is Refs (broader signal — safer
                    // to over-invalidate than miss a branch update). If every
                    // path is noise, drop the batch entirely.
                    let mut kind: Option<EventKind> = None;
                    for ev in &events {
                        if let Some(k) = classify(&ev.path, &repo_root) {
                            kind = Some(match (kind, k) {
                                (Some(EventKind::Refs), _) | (_, EventKind::Refs) => {
                                    EventKind::Refs
                                }
                                _ => EventKind::Workdir,
                            });
                        }
                    }
                    let Some(kind) = kind else {
                        // Entire batch was noise — drop without emitting.
                        return;
                    };
                    if let Err(e) = app.emit(
                        REPO_CHANGED_EVENT,
                        serde_json::json!({ "id": id_for_thread.clone(), "kind": kind }),
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
