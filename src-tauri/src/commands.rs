#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Error {
    message: String,
    detail: String,
}

pub mod config {
    use tauri::Manager;

    use super::Error;
    use crate::{config::Config, ConfigPathState, ConfigState};

    #[tauri::command]
    pub async fn get_config(app: tauri::AppHandle) -> serde_json::Value {
        let config = app.state::<ConfigState>();
        let config = config.lock().await;

        serde_json::to_value(&*config).expect("Failed to parse config to JSON format")
    }

    pub async fn write_config(config_path: &ConfigPathState, config: &Config) -> Result<(), Error> {
        let data = serde_json::to_vec_pretty(&config).map_err(|e| Error {
            message: "設定ファイルのデータに失敗しました。".to_owned(),
            detail: e.to_string(),
        })?;

        async_fs::write(&**config_path, data)
            .await
            .map_err(|e| Error {
                message: "設定ファイルの書き込に失敗しました。".to_owned(),
                detail: e.to_string(),
            })?;

        Ok(())
    }

    #[tauri::command]
    pub async fn save_config(app: tauri::AppHandle, config: Config) -> Result<(), Error> {
        // Save the config to the file.
        let config_path = app.state::<ConfigPathState>();
        write_config(&config_path, &config).await?;

        // Update internal state.
        let state = app.state::<ConfigState>();
        *state.lock().await = config;

        Ok(())
    }
}

pub mod application_window {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};

    use crate::os::application_monitor::get_application_process;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ApplicationWindow {
        pub title: Option<String>,
        pub name: Option<String>,
        pub path: PathBuf,
    }

    #[tauri::command]
    pub async fn get_application_windows() -> Vec<ApplicationWindow> {
        tauri::async_runtime::spawn_blocking(get_application_windows_blocking)
            .await
            .expect("Failed to get application windows")
    }

    fn get_application_windows_blocking() -> Vec<ApplicationWindow> {
        let windows = window_getter::get_windows().expect("Failed to get windows");
        let mut applications = Vec::new();
        let mut added = std::collections::HashSet::new();

        for window in windows {
            let Some(pid) = window.owner_pid().ok() else {
                // For windows, sometimes the PID is not available due to the lack of permissions.
                // So skip such windows.
                continue;
            };

            if std::process::id() == pid as u32 || added.contains(&pid) {
                // If the PID matches the overkami itself, skip it.
                // This is to avoid repetion of application wallpapers to overkami itself.
                continue;
            }
            added.insert(pid);

            let Some(process) = get_application_process(pid as _) else {
                continue;
            };

            let title = window.title().ok().flatten().and_then(|text| {
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            });

            applications.push(ApplicationWindow {
                title,
                name: process.name,
                path: process.path,
            });
        }

        applications
    }
}
