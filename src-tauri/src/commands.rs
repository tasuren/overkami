use anyhow::Context;
use dialog_unwrapper::ErrorDialogUnwrapper;

use crate::data::AppPaths;

#[tauri::command]
#[specta::specta]
pub fn get_app_dir_paths(app: tauri::AppHandle) -> AppPaths {
    AppPaths::new(app)
        .context("Failed to detect the app directory paths.")
        .unwrap_or_dialog()
}
