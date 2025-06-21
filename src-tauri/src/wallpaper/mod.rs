mod application_observer;
mod instance;
mod overlay_manager;

pub use instance::{WallpaperInstance, WallpaperWindows};
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
    use std::collections::HashMap;

    use pollster::FutureExt;
    use smallvec::{smallvec, SmallVec};
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
        let mut pids = HashMap::<usize, SmallVec<[u32; 5]>>::new();

        for app in monitor.get_application_processes() {
            for (i, instance) in instances.iter_mut().enumerate() {
                let config = instance.config();
                let config = config.blocking_lock();

                if app.path == config.application.path {
                    if let Some(pids) = pids.get_mut(&i) {
                        pids.push(app.pid);
                    } else {
                        pids.insert(i, smallvec![app.pid]);
                    }
                }
            }
        }

        // Start window overlays
        for (i, pids) in pids {
            let instance = instances.get_mut(i).unwrap();
            instance.start(pids).block_on();
        }
    }
}
