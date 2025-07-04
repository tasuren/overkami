use tauri::Manager;

use crate::{config::Config, ConfigPathState, ConfigState};

#[tauri::command]
pub async fn get_config(app: tauri::AppHandle) -> serde_json::Value {
    let config = app.state::<ConfigState>();
    let config = config.lock().await;

    serde_json::to_value(&*config).unwrap()
}

pub async fn write_config(config_path: &ConfigPathState, config: &Config) {
    let data = serde_json::to_vec_pretty(&config).expect("Failed to parse config data.");

    async_fs::write(&**config_path, data)
        .await
        .expect("Failed to write config file.")
}

#[tauri::command]
pub async fn save_config(app: tauri::AppHandle, config: Config) {
    log::info!("Saving configuration...");

    // Save the config to the file.
    let config_path = app.state::<ConfigPathState>();
    write_config(&config_path, &config).await;

    // Update internal state.
    let state = app.state::<ConfigState>();
    *state.lock().await = config;
}
