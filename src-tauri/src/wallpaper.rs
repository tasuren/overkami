use tauri::{async_runtime::Mutex, Manager};

use crate::{
    model::{config::Config, WallpaperInstance},
    os::ApplicationProcess,
    wallpaper,
};

pub fn setup_wallpapers(app: &tauri::App) {
    let config_state = app.state::<Mutex<Config>>();
    let config = config_state.blocking_lock();

    let mut instances = Vec::new();
    for wallpaper in config.wallpapers.iter() {
        let instance = WallpaperInstance::new(app.handle().clone(), wallpaper.clone());

        instances.push(instance);
    }

    app.manage(Mutex::new(instances));
}
