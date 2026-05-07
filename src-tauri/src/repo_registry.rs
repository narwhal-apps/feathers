use crate::error::AppError;
use crate::git_core::repo;
use git2::Repository;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

pub type RepoId = String;

pub struct RepoHandle {
    pub id: RepoId,
    pub path: PathBuf, // canonicalized
    pub repo: Mutex<Repository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSummary {
    pub id: RepoId,
    pub name: String, // basename of path
    pub path: String,
}

pub struct RepoRegistry {
    inner: RwLock<HashMap<RepoId, Arc<RepoHandle>>>,
}

impl RepoRegistry {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, path: PathBuf) -> Result<RepoId, AppError> {
        let canonical = path.canonicalize().map_err(|e| AppError::Io {
            message: e.to_string(),
        })?;

        // Reuse if already present (compare by canonicalized path).
        {
            let read = self.inner.read();
            for (id, h) in read.iter() {
                if h.path == canonical {
                    return Ok(id.clone());
                }
            }
        }

        // Open via libgit2 — this both validates and gives us a Repository.
        let r = repo::open(&canonical)?;
        let id = Uuid::new_v4().to_string();
        let handle = Arc::new(RepoHandle {
            id: id.clone(),
            path: canonical,
            repo: Mutex::new(r),
        });
        self.inner.write().insert(id.clone(), handle);
        Ok(id)
    }

    pub fn get(&self, id: &str) -> Result<Arc<RepoHandle>, AppError> {
        self.inner
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::RepoNotFound { id: id.to_string() })
    }

    pub fn remove(&self, id: &str) -> Result<(), AppError> {
        self.inner
            .write()
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| AppError::RepoNotFound { id: id.to_string() })
    }

    pub fn list(&self) -> Vec<RepoSummary> {
        self.inner
            .read()
            .values()
            .map(|h| RepoSummary {
                id: h.id.clone(),
                name: h
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                path: h.path.to_string_lossy().to_string(),
            })
            .collect()
    }

    /// Add a repo by path *without* validating it points to a working tree —
    /// used at startup when restoring known repos: we want to show them in
    /// the UI even if they've been deleted, then handle errors lazily.
    /// (NOT used in M2: we strict-validate on startup. Stub for M3 if needed.)
    #[allow(dead_code)]
    pub fn paths(&self) -> Vec<PathBuf> {
        self.inner.read().values().map(|h| h.path.clone()).collect()
    }
}

impl Default for RepoRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn short_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}
