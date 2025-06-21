mod application_observer;
mod instance;
mod overlay_manager;

pub use instance::{WallpaperConfig, WallpaperInstance, WallpaperWindows};
pub use overlay_manager::OverlayManager;
pub use setup::setup_wallpapers;
pub use state::WallpaperInstanceState;

mod state {
    use tauri::{async_runtime::Mutex, Manager};

    use crate::wallpaper::WallpaperInstance;

    pub type WallpaperInstanceState = Mutex<Vec<WallpaperInstance>>;

    pub fn set_wallpaper_instance_state(app: &tauri::AppHandle, instances: Vec<WallpaperInstance>) {
        app.manage(Mutex::new(instances));
    }
}

mod setup {
    use pollster::FutureExt;
    use tauri::{async_runtime, App, AppHandle, Manager};

    use super::{state::set_wallpaper_instance_state, WallpaperInstance, WallpaperInstanceState};
    use crate::{config::ConfigState, os::ApplicationMonitor};

    pub fn setup_wallpapers(app: &App) {
        let config_state = app.state::<ConfigState>();
        let config = config_state.blocking_lock();

        let mut instances = Vec::new();
        for wallpaper in config.wallpapers.iter() {
            let instance = WallpaperInstance::new(app.handle().clone(), wallpaper.clone());

            instances.push(instance);
        }

        set_wallpaper_instance_state(app.handle(), instances);

        let app = app.handle().clone();
        async_runtime::spawn_blocking(move || start_wallpapers(app));
    }

    fn start_wallpapers(app: AppHandle) {
        let state = app.state::<WallpaperInstanceState>();
        let mut instances = state.blocking_lock();

        let monitor = ApplicationMonitor::new();
        let windows = window_getter::get_windows().expect("Failed to get windows");

        for window in windows {
            let Some(pid) = window.owner_pid().ok() else {
                continue;
            };

            for instance in instances.iter_mut() {
                if let Some(app) = monitor.get_application_process(pid as _) {
                    {
                        let config = instance.config();
                        let config = config.blocking_lock();

                        if app.path != config.application.path {
                            continue;
                        }
                    }

                    instance.start(app).block_on();
                };
            }
        }
    }
}
