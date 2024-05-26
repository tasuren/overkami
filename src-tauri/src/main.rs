// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

pub mod logic;
pub mod presentation;
pub mod sys;

use sys::Core;

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window_manager = presentation::setup_wallpaper_windows(app);
    app.manage(Core { windows: window_manager });
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
