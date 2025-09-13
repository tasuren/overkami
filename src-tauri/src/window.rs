use tauri::Manager;

pub fn setup_window(app: &tauri::App) {
    let main_window = app.get_webview_window("main").unwrap();
    
    crate::os::platform_impl::custom_feature::
}
