use std::collections::HashMap;

use tauri::{
    async_runtime::{self, Mutex},
    Manager,
};
use uuid::Uuid;

use crate::{event_manager::EventManager, wallpaper::wallpaper_host::WallpaperHost, ConfigState};

pub type WallpaperHosts = Mutex<HashMap<Uuid, WallpaperHost>>;

pub fn setup_wallpapers(app: &tauri::App) {
    log::info!("Initializing wallpaper hosts...");

    let hosts = async_runtime::block_on({
        let app = app.handle().clone();

        async move {
            let config = app.state::<ConfigState>();
            let config = config.lock().await;

            let mut hosts = HashMap::new();

            for (id, wallpaper) in config.wallpapers.iter() {
                let host = WallpaperHost::new(*id, wallpaper.clone(), app.clone()).await;
                hosts.insert(*id, host);
            }

            hosts
        }
    });

    app.manage(Mutex::new(hosts));

    setup_wallpaper_management(app);
}

/// Sets up the event listeners for wallpaper addition or removal.
pub fn setup_wallpaper_management(app: &tauri::App) {
    let event_manager = app.state::<EventManager>();

    event_manager.listen_add_wallpaper({
        let app = app.handle().clone();

        move |data| {
            let app = app.clone();

            async_runtime::spawn(async move {
                log::info!("Adding new wallpaper host for ID: {}", data.id);

                let host = WallpaperHost::new(data.id, data.wallpaper, app.clone()).await;
                app.state::<WallpaperHosts>()
                    .lock()
                    .await
                    .insert(data.id, host);
            });
        }
    });

    event_manager.listen_remove_wallpaper({
        let app = app.handle().clone();

        move |id| {
            let app = app.clone();

            async_runtime::spawn(async move {
                log::info!("Removing wallpaper host for ID: {}", id);

                let hosts = app.state::<WallpaperHosts>();
                let mut hosts = hosts.lock().await;

                if let Some(host) = hosts.remove(&id) {
                    host.stop().await;
                };
            });
        }
    });
}
