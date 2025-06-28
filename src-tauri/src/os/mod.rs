pub mod application_monitor;
pub mod application_observer;
pub mod platform_impl;

pub use platform_impl::{WebviewWindowPlatformExt, WindowPlatformExt};

pub mod windows {
    use window_getter::Window;

    pub async fn get_windows() -> Vec<Window> {
        tauri::async_runtime::spawn_blocking(|| {
            window_getter::get_windows().expect("Failed to get windows")
        })
        .await
        .expect("Failed to spawn getting windows")
    }
}
