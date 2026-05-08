use crate::error::AppError;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const SETTINGS_LABEL: &str = "settings";

/// Open the Settings window. If it already exists, focus + raise it instead
/// of spawning a new one — this matches the macOS prefs-window contract.
#[tauri::command]
pub async fn open_settings_window(app: AppHandle) -> Result<(), AppError> {
    if let Some(existing) = app.get_webview_window(SETTINGS_LABEL) {
        existing.show().ok();
        existing.set_focus().ok();
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, SETTINGS_LABEL, WebviewUrl::App("/settings".into()))
        .title("Feathers Settings")
        .inner_size(720.0, 520.0)
        .min_inner_size(600.0, 440.0)
        .resizable(true)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        .traffic_light_position(tauri::LogicalPosition::new(16.0, 20.0))
        .build()
        .map_err(|e| AppError::Io { message: format!("create settings window: {e}") })?;
    Ok(())
}
