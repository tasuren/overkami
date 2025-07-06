use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub use payload::*;

use crate::{
    commands::CommandError, config::Wallpaper, wallpaper::WallpaperHostsState, ConfigState,
};

#[tauri::command]
pub async fn apply_wallpaper(
    app: AppHandle,
    id: Uuid,
    payload: payload::ApplyWallpaper,
) -> Result<(), CommandError> {
    log::info!("Apply wallpaper `{id}`");

    let config = app.state::<ConfigState>();
    let mut config = config.lock().await;
    let Some(wallpaper) = config.wallpapers.get_mut(&id) else {
        return Err(CommandError {
            code: "wallpaper_not_found".to_owned(),
            detail: None,
        });
    };
    log::debug!("current: {wallpaper:?}");

    // Update the wallpaper configuration with the provided payload.
    let old_wallpaper = wallpaper.clone();
    update_wallpaper_config(wallpaper, payload.clone()).await;

    // Sync the updated wallpaper configuration to wallpaper overlays.
    let hosts = app.state::<WallpaperHostsState>();
    let hosts = hosts.lock().await;

    if let Some(host) = hosts.get(&id) {
        println!("old: {old_wallpaper:?}");
        println!("new: {payload:?}");
        host.apply_wallpaper(old_wallpaper, payload).await;

        Ok(())
    } else {
        Err(CommandError {
            code: "wallpaper_not_found".to_owned(),
            detail: None,
        })
    }
}

async fn update_wallpaper_config(wallpaper: &mut Wallpaper, payload: payload::ApplyWallpaper) {
    if let Some(name) = payload.name {
        wallpaper.name = name;
    }

    if let Some(application_path) = payload.application_path {
        println!("{}", application_path.display());
        wallpaper.application_path = application_path;
    }

    if let Some(filters) = payload.filters {
        wallpaper.filters = filters;
    }

    if let Some(opacity) = payload.opacity {
        wallpaper.opacity = opacity;
    }

    if let Some(source) = payload.source {
        wallpaper.source = source;
    }
}

#[tauri::command]
pub async fn add_wallpaper(app: AppHandle, id: Uuid, payload: payload::AddWallpaper) {
    log::info!("Add new wallpaper");
    log::debug!("Payload: {payload:#?}");

    crate::wallpaper::add_wallpaper(app, id, payload).await;
}

#[tauri::command]
pub async fn remove_wallpaper(app: AppHandle, id: Uuid) {
    log::info!("Remove wallpaper: id = {id}");

    crate::wallpaper::remove_wallpaper(&app, id).await;
}

mod payload {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};

    use crate::config::{Filter, Wallpaper, WallpaperSource};

    /// Represents the payload for applying wallpaper settings.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ApplyWallpaper {
        pub name: Option<String>,
        pub application_path: Option<PathBuf>,
        pub filters: Option<Vec<Filter>>,
        pub opacity: Option<f64>,
        pub source: Option<WallpaperSource>,
    }

    /// Represents the payload for adding a new wallpaper configuration.
    pub type AddWallpaper = Wallpaper;
}
