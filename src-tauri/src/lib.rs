mod commands;
mod config;
mod model;
mod os;
mod service;
mod utils;

fn setup(app: &tauri::App) {
    config::setup_config(app);
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
        .invoke_handler(tauri::generate_handler![commands::get_application_windows])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
