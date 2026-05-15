use crate::error::AppError;
use crate::repo_registry::RepoRegistry;
use std::path::Path;
use std::process::Command;
use tauri::State;

/// Open the active repo's working directory in the user's preferred editor.
/// Tries known macOS editor apps in order; falls back to the OS default
/// folder handler (Finder) when none are installed.
#[tauri::command]
pub async fn repo_open_in_editor(
    id: String,
    registry: State<'_, RepoRegistry>,
) -> Result<String, AppError> {
    let path = {
        let handle = registry.get(&id)?;
        handle.path.clone()
    };
    open_path_in_editor(&path)
}

fn open_path_in_editor(path: &Path) -> Result<String, AppError> {
    // Priority list — a quick check matches what most macOS devs have.
    // Add to this as needed; first match wins.
    const CANDIDATES: &[&str] = &[
        "Visual Studio Code",
        "Cursor",
        "Zed",
        "Sublime Text",
        "WebStorm",
        "IntelliJ IDEA",
        "PyCharm",
        "RubyMine",
        "GoLand",
        "PhpStorm",
        "RustRover",
        "Nova",
        "Xcode",
    ];

    for app in CANDIDATES {
        if app_installed(app) {
            run_open_a(app, path)?;
            return Ok((*app).to_string());
        }
    }

    // No known editor → fall back to the system default (Finder).
    run_open(path)?;
    Ok("Finder".to_string())
}

fn app_installed(app: &str) -> bool {
    let bundle = format!("{app}.app");
    if Path::new("/Applications").join(&bundle).exists() {
        return true;
    }
    if let Some(home) = home::home_dir() {
        if home.join("Applications").join(&bundle).exists() {
            return true;
        }
    }
    false
}

fn run_open_a(app: &str, path: &Path) -> Result<(), AppError> {
    Command::new("open")
        .arg("-a")
        .arg(app)
        .arg(path)
        .spawn()
        .map_err(|e| AppError::Io {
            message: format!("failed to launch {app}: {e}"),
        })?;
    Ok(())
}

fn run_open(path: &Path) -> Result<(), AppError> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|e| AppError::Io {
            message: format!("failed to open {}: {e}", path.display()),
        })?;
    Ok(())
}
