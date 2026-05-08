use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeName {
    Dark,
    Light,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    /// `None` means "follow OS preference". `Some(name)` pins the theme.
    pub theme_override: Option<ThemeName>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub schema: u32,
    pub known_repos: Vec<String>,
    #[serde(default)]
    pub settings: AppSettings,
}

impl AppConfig {
    pub fn current_schema() -> u32 {
        2
    }

    fn fresh() -> Self {
        Self {
            schema: Self::current_schema(),
            known_repos: vec![],
            settings: AppSettings::default(),
        }
    }
}

pub trait ConfigStore: Send + Sync {
    fn load(&self) -> Result<AppConfig, AppError>;
    fn save(&self, cfg: &AppConfig) -> Result<(), AppError>;
}

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
            return Ok(AppConfig::fresh());
        }
        let raw = std::fs::read_to_string(&self.path)?;
        let mut cfg: AppConfig = serde_json::from_str(&raw).map_err(|e| AppError::Io {
            message: format!("config parse: {e}"),
        })?;
        if cfg.schema < AppConfig::current_schema() {
            cfg = migrate(cfg)?;
            self.save(&cfg)?;
        }
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

fn migrate(mut cfg: AppConfig) -> Result<AppConfig, AppError> {
    while cfg.schema < AppConfig::current_schema() {
        match cfg.schema {
            0 | 1 => {
                cfg.schema = 2;
            }
            other => {
                return Err(AppError::Io {
                    message: format!("unknown config schema {other}"),
                });
            }
        }
    }
    Ok(cfg)
}
