// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;

pub mod logic;
pub mod presentation;
pub mod sys;

use sys::Core;

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create a custom titlebar for main window.
    let main_window = app.get_webview_window("main").unwrap();
    main_window.create_overlay_titlebar().unwrap();
    // Set a custom inset to the traffic lights.
    #[cfg(target_os = "macos")]
    main_window.set_traffic_lights_inset(20., 20.).unwrap();

    let window_manager = presentation::setup_wallpaper_windows(app);
    app.manage(Core {
        windows: window_manager,
    });
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_decorum::init())
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
