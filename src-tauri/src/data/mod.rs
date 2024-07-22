use std::path::PathBuf;

use serde::Serialize;
use tauri::Manager;

pub mod profile;

pub mod error {
    use std::path::PathBuf;

    #[derive(Debug, thiserror::Error)]
    pub enum FileError {
        #[error("Failed to operate the file: {path}")]
        IO {
            source: std::io::Error,
            path: PathBuf,
        },
        #[error("Something went wrong.")]
        Other(#[from] anyhow::Error),
    }
}

#[derive(Serialize, specta::Type)]
pub struct AppPaths {
    app_config_dir: PathBuf,
}

impl Default for AppPaths {
    fn default() -> Self {
        Self {
            app_config_dir: PathBuf::from(".debug_app_dir/config"),
        }
    }
}

impl AppPaths {
    pub fn new(app: tauri::AppHandle) -> Result<Self, tauri::Error> {
        if cfg!(debug_assertion) {
            Ok(Self {
                app_config_dir: app.path().app_config_dir()?,
            })
        } else {
            Ok(Self::default())
        }
    }
}
