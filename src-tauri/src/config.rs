use std::{ path::PathBuf};

use tauri::{async_runtime::Mutex, App, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::model::config::Config;

pub fn setup_config(app: &App) {
    let path = app.path();

    let config_path = if cfg!(debug_assertions) {
        PathBuf::from("mock-env/config")
    } else {
        path.app_config_dir()
            .expect("Failed to get app config directory")
    };

    if !config_path.exists() {
        if let Err(error) = std::fs::create_dir_all(&config_path) {
            failed_to_prepare_config_dir(app, error.to_string());
        };
    }

    let config_file = config_path.join("config.json");

    let config = if config_file.exists() {
        let raw = match std::fs::read(config_file) {
            Ok(raw) => raw,
            Err(error) => failed_to_read_config_file(app, error.to_string()),
        };

        match serde_json::from_slice(&raw) {
            Ok(config) => config,
            Err(error) => failed_to_parse_config_file(app, error.to_string()),
        }
    } else {
        Config::default()
    };

    app.manage(Mutex::new(config));
}

fn failed_to_prepare_config_dir(app: &App, error: String) -> ! {
    app.dialog()
        .message(format!(
            "設定ファイルを配置するフォルダの作成に失敗しました。\n詳細: {error}"
        ))
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();

    std::process::exit(1);
}

fn failed_to_read_config_file(app: &App, error: String) -> ! {
    app.dialog()
        .message(format!(
            "設定ファイルの読み込みに失敗しました。\n詳細: {error}"
        ))
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();

    std::process::exit(1);
}

fn failed_to_parse_config_file(app: &App, error: String) -> ! {
    app.dialog()
        .message(format!(
            "設定ファイルのデータ構造が不適切です。\n詳細: {error}"
        ))
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();

    std::process::exit(1);
}
