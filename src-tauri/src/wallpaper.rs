use tauri::{
    async_runtime::{self, Mutex},
    App, Listener, Manager,
};

use crate::{
    model::{config::Config, WallpaperInstance},
    os::{ApplicationMonitor, ApplicationProcess},
    wallpaper,
};

pub fn setup_wallpapers(app: &App) {
    let config_state = app.state::<Mutex<Config>>();
    let config = config_state.blocking_lock();

    let mut instances = Vec::new();
    for wallpaper in config.wallpapers.iter() {
        let instance = WallpaperInstance::new(app.handle().clone(), wallpaper.clone());

        instances.push(instance);
    }

    app.manage(Mutex::new(instances));

    // Initilalize wallpaper windows.
    async_runtime::spawn({
        let app = app.handle().clone();

        async move {
            let state = app.state::<Mutex<Vec<WallpaperInstance>>>();
            let mut instances = state.lock().await;

            let monitor = async_runtime::spawn_blocking(ApplicationMonitor::new)
                .await
                .unwrap();

            let windows = window_getter::get_windows().expect("Failed to get windows");
            for window in windows {
                let Some(pid) = window.owner_pid().ok() else {
                    continue;
                };

                for instance in instances.iter_mut() {
                    if let Some(app) = monitor.get_application_process(pid as _) {
                        instance.on_new_app(app).await;
                    };
                }
            }
        }
    });

    // Set up window observer.
    setup_window_observer(app);
}

fn setup_window_observer(app: &App) {}
