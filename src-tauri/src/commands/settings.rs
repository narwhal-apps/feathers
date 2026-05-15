use crate::error::AppError;
use crate::persistence::store::{AppSettings, ConfigStore, ThemeName};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

const SETTINGS_CHANGED_EVENT: &str = "settings_changed";

#[tauri::command]
pub async fn settings_get(
    store: State<'_, Arc<dyn ConfigStore>>,
) -> Result<AppSettings, AppError> {
    let cfg = store.inner().load()?;
    Ok(cfg.settings)
}

#[tauri::command]
pub async fn settings_set_theme(
    theme: Option<ThemeName>,
    app: AppHandle,
    store: State<'_, Arc<dyn ConfigStore>>,
) -> Result<(), AppError> {
    let mut cfg = store.inner().load()?;
    cfg.settings.theme_override = theme;
    store.inner().save(&cfg)?;
    if let Err(e) = app.emit(SETTINGS_CHANGED_EVENT, ()) {
        tracing::warn!("failed to emit {SETTINGS_CHANGED_EVENT}: {e}");
    }
    Ok(())
}

/// Persist the canonical path of the most-recently-active repo so the
/// next launch can auto-open it. `None` clears the value (e.g. user
/// closed every repo). Path-based, not id-based — registry ids are
/// minted fresh on each process start, so an id wouldn't survive a
/// restart. No event emit — only this window cares.
#[tauri::command]
pub async fn settings_set_last_active_repo_path(
    path: Option<String>,
    store: State<'_, Arc<dyn ConfigStore>>,
) -> Result<(), AppError> {
    let mut cfg = store.inner().load()?;
    cfg.settings.last_active_repo_path = path;
    store.inner().save(&cfg)?;
    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitIdentity {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Read user.name / user.email from a global gitconfig rooted at `home`.
/// Used so tests can sandbox the lookup; production callers pass `home::home_dir()`.
pub fn read_git_identity_from(home: &Path) -> Result<GitIdentity, AppError> {
    let path = home.join(".gitconfig");
    if !path.exists() {
        return Ok(GitIdentity::default());
    }
    let cfg = git2::Config::open(&path).map_err(AppError::from)?;
    let name = cfg.get_string("user.name").ok().filter(|s| !s.is_empty());
    let email = cfg.get_string("user.email").ok().filter(|s| !s.is_empty());
    Ok(GitIdentity { name, email })
}

/// Write user.name / user.email to a global gitconfig rooted at `home`.
/// Empty strings unset (delete) the keys. Both must be empty or both
/// non-empty — half-set identities are rejected. Email must look like an
/// address (contain `@` and a `.` after it).
pub fn write_git_identity_to(home: &Path, name: &str, email: &str) -> Result<(), AppError> {
    let name = name.trim();
    let email = email.trim();
    if name.is_empty() != email.is_empty() {
        return Err(AppError::Git {
            message: "git identity needs both name and email (or both empty to clear)".into(),
        });
    }
    if !email.is_empty() {
        if let Some(at) = email.find('@') {
            if !email[at + 1..].contains('.') {
                return Err(AppError::Git { message: "email looks invalid".into() });
            }
        } else {
            return Err(AppError::Git { message: "email looks invalid".into() });
        }
    }
    let path = home.join(".gitconfig");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut cfg = git2::Config::open(&path).or_else(|_| {
        // Creating an empty file lets git2 open it for writing.
        std::fs::write(&path, "")?;
        git2::Config::open(&path).map_err(AppError::from)
    })?;
    if name.is_empty() {
        let _ = cfg.remove("user.name");
        let _ = cfg.remove("user.email");
    } else {
        cfg.set_str("user.name", name)?;
        cfg.set_str("user.email", email)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn settings_get_git_identity() -> Result<GitIdentity, AppError> {
    let home = home::home_dir().ok_or_else(|| AppError::Io {
        message: "couldn't resolve home directory".into(),
    })?;
    read_git_identity_from(&home)
}

#[tauri::command]
pub async fn settings_set_git_identity(name: String, email: String) -> Result<(), AppError> {
    let home = home::home_dir().ok_or_else(|| AppError::Io {
        message: "couldn't resolve home directory".into(),
    })?;
    write_git_identity_to(&home, &name, &email)
}
