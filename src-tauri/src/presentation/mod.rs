mod window_manager;

pub use window_manager::WindowManager;

pub fn setup_wallpaper_windows(app: &mut tauri::App) -> WindowManager {
    WindowManager::new(app)
}
