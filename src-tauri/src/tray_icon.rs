use tauri::{menu::MenuBuilder, tray::TrayIconBuilder, Manager};

pub fn setup_tray_icon(app: &mut tauri::App) {
    let window = app.get_webview_window("main").unwrap();
    window.on_window_event({
        let window = window.clone();

        move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().expect("Failed to hide main window");
            }
        }
    });

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let menu = MenuBuilder::new(app)
        .about_with_text("overkamiについて", None)
        .separator()
        .text("settings", "設定")
        .quit_with_text("overkamiを終了する")
        .build()
        .expect("Failed to create tray icon menu");

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |_app, event| match event.id().as_ref() {
            "about" => {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            "settings" => {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            _ => {}
        })
        .build(app)
        .expect("Failed to create tray icon");
}
