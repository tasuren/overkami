mod commands;
mod model;
mod os;
mod service;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::get_application_windows])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
