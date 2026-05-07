use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub schema: u32,
    pub known_repos: Vec<String>,
}

impl AppConfig {
    pub fn current_schema() -> u32 {
        1
    }
}

/// Trait so commands can call this without a Tauri runtime in tests.
pub trait ConfigStore: Send + Sync {
    fn load(&self) -> Result<AppConfig, AppError>;
    fn save(&self, cfg: &AppConfig) -> Result<(), AppError>;
}

/// File-backed implementation; used at runtime via `tauri-plugin-store`'s
/// resolved app data directory. We bypass the plugin and write JSON ourselves
/// because `tauri-plugin-store`'s API is async and requires a Runtime —
/// fine for FE access, awkward for backend startup. Both writers can coexist
/// since they target the same file.
pub struct FileStore {
    pub path: PathBuf,
}

impl FileStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ConfigStore for FileStore {
    fn load(&self) -> Result<AppConfig, AppError> {
        if !self.path.exists() {
            return Ok(AppConfig {
                schema: AppConfig::current_schema(),
                known_repos: vec![],
            });
        }
        let raw = std::fs::read_to_string(&self.path)?;
        let cfg: AppConfig = serde_json::from_str(&raw).map_err(|e| AppError::Io {
            message: format!("config parse: {e}"),
        })?;
        Ok(cfg)
    }

    fn save(&self, cfg: &AppConfig) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(cfg).map_err(|e| AppError::Io {
            message: format!("config encode: {e}"),
        })?;
        std::fs::write(&self.path, raw)?;
        Ok(())
    }
}
