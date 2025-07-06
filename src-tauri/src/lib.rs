mod commands;
mod config;
mod os;
mod tray_icon;
mod utils;
mod wallpaper;

pub use config::state::{ConfigPathState, ConfigState};

fn setup(app: &mut tauri::App) {
    log::info!("Starting overkami...");
    panic_hook::setup_panic_hook(app);

    os::application_monitor::auto_refresh::start().unwrap();

    config::setup_config(app);
    wallpaper::setup_wallpapers(app);

    tray_icon::setup_tray_icon(app);
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
            commands::os::get_application_windows,
            commands::sync::apply_wallpaper,
            commands::sync::add_wallpaper,
            commands::sync::remove_wallpaper,
            #[cfg(target_os = "macos")]
            commands::os::platform_custom_feature::set_document_edited
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
                .show(|_| {});

            app.exit(1);

            default_panic(info);
            log::info!("You can use `RUST_BACKTRACE=full` to see the full backtrace.");
        }));
    }
}
