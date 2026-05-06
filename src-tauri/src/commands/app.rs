/// Returns the Cargo package version. Used as the IPC smoke test.
pub fn app_version_inner() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn app_version() -> String {
    app_version_inner()
}
