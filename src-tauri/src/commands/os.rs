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

        let title =
            window.title().ok().flatten().and_then(
                |text| {
                    if text.is_empty() {
                        None
                    } else {
                        Some(text)
                    }
                },
            );

        applications.push(ApplicationWindow {
            title,
            name: process.name,
            path: process.path,
        });
    }

    applications
}

pub mod platform_custom_feature {
    #[cfg(target_os = "macos")]
    #[tauri::command]
    pub fn set_document_edited(window: tauri::WebviewWindow, edited: bool) {
        #[cfg(target_os = "macos")]
        crate::os::platform_impl::custom_feature::set_document_edited(window, edited);
    }
}
