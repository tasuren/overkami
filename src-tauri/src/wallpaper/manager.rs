use std::collections::HashMap;

use tauri::{
    AppHandle, Manager,
    async_runtime::{self, Mutex},
};
use uuid::Uuid;

use crate::{ConfigState, commands::sync::AddWallpaper, wallpaper::wallpaper_host::WallpaperHost};

pub type WallpaperHostsState = Mutex<HashMap<Uuid, WallpaperHost>>;

pub fn setup_wallpapers(app: &tauri::App) {
    log::info!("Initializing wallpaper hosts...");

    let hosts = async_runtime::block_on({
        let app = app.handle().clone();

        async move {
            let config = app.state::<ConfigState>();
            let config = config.lock().await;

            let mut hosts = HashMap::new();

            for (id, wallpaper) in config.wallpapers.iter() {
                let host = WallpaperHost::new(app.clone(), *id, wallpaper.clone()).await;
                hosts.insert(*id, host);
            }

            hosts
        }
    });

    app.manage(Mutex::new(hosts));
}

pub async fn add_wallpaper(app: AppHandle, id: Uuid, payload: AddWallpaper) {
    log::info!("Adding new wallpaper host for ID: {id}");

    let host = WallpaperHost::new(app.clone(), id, payload).await;
    app.state::<WallpaperHostsState>()
        .lock()
        .await
        .insert(id, host);
}

pub async fn remove_wallpaper(app: &AppHandle, id: Uuid) {
    log::info!("Removing wallpaper host for ID: {id}");

    let hosts = app.state::<WallpaperHostsState>();
    let mut hosts = hosts.lock().await;

    if let Some(host) = hosts.remove(&id) {
        host.stop().await;
    };
}
