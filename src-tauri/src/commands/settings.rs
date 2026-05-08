use crate::error::AppError;
use crate::persistence::store::{AppSettings, ConfigStore, ThemeName};
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
