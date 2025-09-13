use tauri::{menu::MenuBuilder, tray::TrayIconBuilder, Manager};

pub fn setup_tray_icon(app: &mut tauri::App) {
    let window = app.get_webview_window("main").unwrap();
    window.on_window_event({
        let window = window.clone();
        let app = app.handle().clone();

        move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                #[cfg(target_os = "macos")]
                app.set_dock_visibility(false).unwrap();

                api.prevent_close();
                window.hide().unwrap();
            }
        }
    });

    let menu = MenuBuilder::new(app)
        .text("settings", "設定")
        .quit_with_text("overkamiを終了する")
        .build()
        .expect("Failed to create tray icon menu");

    #[cfg(target_os = "macos")]
    let icon = tauri::include_image!("icons/MacTrayIcon.png");
    #[cfg(not(target_os = "macos"))]
    let icon = app.default_window_icon().unwrap().clone();

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        .menu(&menu)
        .on_menu_event(move |app, event| {
            if event.id().as_ref() == "settings" {
                window.show().unwrap();
                window.set_focus().unwrap();

                #[cfg(target_os = "macos")]
                app.set_dock_visibility(true).unwrap();
            }
        })
        .build(app)
        .expect("Failed to create tray icon");
}
