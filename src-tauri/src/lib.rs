mod commands;
mod config;
mod os;
mod tray_icon;
mod utils;
mod wallpaper;

pub use config::state::{ConfigPathState, ConfigState};
use tauri::Manager;

fn setup(app: &mut tauri::App) {
    log::info!("Starting overkami...");

    os::application_monitor::auto_refresh::start().unwrap();

    config::setup_config(app);
    wallpaper::setup_wallpapers(app);

    tray_icon::setup_tray_icon(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            setup(app);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::save_config,
            commands::os::get_application_windows,
            commands::sync::apply_wallpaper,
            commands::sync::add_wallpaper,
            commands::sync::remove_wallpaper,
            #[cfg(target_os = "macos")]
            commands::os::platform_custom_feature::set_document_edited
        ])
        .build(tauri::generate_context!())
        .expect("Failed to run tauri application");

    // TODO: 初回起動時のみ↓を実行しない。（ドックにアイコンを表示する。）
    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
        app.set_dock_visibility(false);

        // TODO: CGRequestScreenCaptureAccessを実施
    }

    app.run(|app, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            app.get_webview_window("main").unwrap().set_focus().unwrap();
        }
    });
}
