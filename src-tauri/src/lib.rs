mod commands;
mod config;
mod event_manager;
mod os;
mod tray_icon;
mod utils;
mod wallpaper;

pub use config::state::{ConfigPathState, ConfigState};
pub use event_manager::state::EventManagerState;

fn setup(app: &mut tauri::App) {
    os::application_monitor::auto_refresh::start();

    event_manager::setup_event_manager(app);
    config::setup_config(app);
    wallpaper::setup_wallpapers(app);

    tray_icon::setup_tray_icon(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            setup(app);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::save_config,
            commands::application_window::get_application_windows
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Tauri application");
}
