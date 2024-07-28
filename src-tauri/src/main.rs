// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, WindowEvent};
use tauri_plugin_decorum::WebviewWindowExt;

pub mod commands;
pub mod data;
pub mod logic;
pub mod presentation;
pub mod sys;

use sys::Core;

pub mod prelude {
    pub use anyhow::{Context as _, Result};
}

fn apply_main_window_decoration(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    window.set_traffic_lights_inset(20., 20.).unwrap();
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create a custom titlebar for main window.
    let main_window = app.get_webview_window("main").unwrap();
    // main_window.create_overlay_titlebar().unwrap();
    // Set a custom inset to the traffic lights.
    apply_main_window_decoration(&main_window);

    #[cfg(debug_assertions)]
    main_window.open_devtools();

    let window_manager = presentation::setup_wallpaper_windows(app);
    app.manage(Core {
        windows: window_manager,
    });
    Ok(())
}

fn main() {
    let invoke_handler = {
        let builder = tauri_specta::ts::builder()
            .commands(tauri_specta::collect_commands![commands::get_app_dir_paths]);

        #[cfg(debug_assertions)]
        let builder = builder.path("../src/binding.ts");

        builder.build().unwrap()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_decorum::init())
        .setup(setup)
        .invoke_handler(invoke_handler)
        .on_window_event(|window, we| {
            let apply_webview_window_decoration =
                || apply_main_window_decoration(&window.get_webview_window("main").unwrap());

            match *we {
                WindowEvent::Resized(_) => apply_webview_window_decoration(),
                WindowEvent::ThemeChanged(_) => apply_webview_window_decoration(),
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
