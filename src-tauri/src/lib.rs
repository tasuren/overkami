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

    wallpaper::setup_wallpapers(app);
    tray_icon::setup_tray_icon(app);

    if app
        .state::<ConfigState>()
        .blocking_lock()
        .open_window_on_startup
    {
        app.get_window("main").unwrap().show().unwrap();
    }
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

    config::setup_config(&app);

    #[cfg(target_os = "macos")]
    {
        if app
            .state::<ConfigState>()
            .blocking_lock()
            .open_window_on_startup
        {
            app.set_activation_policy(tauri::ActivationPolicy::Regular);
            app.set_dock_visibility(true);
        } else {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            app.set_dock_visibility(false);
        }

        use tauri_plugin_dialog::DialogExt;
        use window_getter::platform_impl::macos::permission;

        if !permission::has_screen_capture_access() {
            use tauri_plugin_dialog::MessageDialogButtons;

            app.dialog()
                .message(
                    "ウィンドウ情報を取得するのに画面収録の許可が必要です。\n\
                    許可した後、アプリを再起動してください。",
                )
                .title("画面収録の許可が必要です。")
                .buttons(MessageDialogButtons::Ok)
                .show(|_| {
                    permission::request_screen_capture_access();
                    std::process::exit(0);
                });
        }
    }

    app.run(|app, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            app.get_webview_window("main").unwrap().set_focus().unwrap();
        }
    });
}
