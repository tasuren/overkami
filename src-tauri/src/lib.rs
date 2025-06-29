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
    log::info!("Starting overkami...");

    panic_hook::setup_panic_hook(app);

    os::application_monitor::auto_refresh::start().unwrap();

    event_manager::setup_event_manager(app);
    config::setup_config(app);
    wallpaper::setup_wallpapers(app);

    tray_icon::setup_tray_icon(app);

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level_for("tao", log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            setup(app);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::save_config,
            commands::application_window::get_application_windows,
            #[cfg(target_os = "macos")]
            commands::platform_custom_feature::set_document_edited
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Tauri application");
}

mod panic_hook {
    use tauri_plugin_dialog::DialogExt;

    pub fn setup_panic_hook(app: &tauri::App) {
        log_panics::init();

        let default_panic = std::panic::take_hook();
        let app = app.handle().clone();

        std::panic::set_hook(Box::new(move |info| {
            app.dialog()
                .message(
                    "overkamiにて、致命的なエラーが発生しましたので、overkamiを終了します。\n\
                    エラーの詳細はログに出力されます。",
                )
                .title("致命的なエラーが発生")
                .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                .show(|_| std::process::exit(1));

            default_panic(info);
            log::info!("You can use `RUST_BACKTRACE=full` to see the full backtrace.");
        }));
    }
}
