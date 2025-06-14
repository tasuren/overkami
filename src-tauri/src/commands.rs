use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::os::ApplicationMonitor;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationWindow {
    pub title: Option<String>,
    pub name: Option<String>,
    pub path: PathBuf,
}

#[tauri::command]
pub async fn get_application_windows() -> Vec<ApplicationWindow> {
    tauri::async_runtime::spawn_blocking(|| {
        let windows = window_getter::get_windows().expect("Failed to get windows");
        let mut applications = Vec::new();

        for window in windows {
            let Some(pid) = window.owner_pid().ok() else {
                // For windows, sometimes the PID is not available due to the lack of permissions.
                // So skip such windows.
                continue;
            };

            let Some(process) = ApplicationMonitor::new().get_application_process(pid as _) else {
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
    })
    .await
    .expect("Failed to get application windows")
}
