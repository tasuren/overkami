use tauri::{AppHandle, Emitter, Listener};
use uuid::Uuid;

use crate::event_manager::payload::AddWallpaper;

pub struct EventManager {
    app: AppHandle,
}

impl EventManager {
    fn new(app: AppHandle) -> Self {
        Self { app }
    }

    const APPLY_WALLPAPER_EVENT: &str = "apply-wallpaper";

    pub fn emit_apply_wallpaper(&self, data: payload::ApplyWallpaper) -> anyhow::Result<()> {
        Ok(self.app.emit(Self::APPLY_WALLPAPER_EVENT, data)?)
    }

    pub fn listen_apply_wallpaper<F: Fn(payload::ApplyWallpaper) + Send + 'static>(
        &self,
        callback: F,
    ) -> u32 {
        self.app.listen(Self::APPLY_WALLPAPER_EVENT, move |event| {
            let data: payload::ApplyWallpaper = serde_json::from_str(event.payload()).unwrap();

            callback(data);
        })
    }

    const ADD_WALLPAPER_EVENT: &str = "add-wallpaper";

    pub fn emit_add_wallpaper(&self, data: AddWallpaper) -> anyhow::Result<()> {
        Ok(self.app.emit(Self::ADD_WALLPAPER_EVENT, data)?)
    }

    pub fn listen_add_wallpaper<F: Fn(AddWallpaper) + Send + 'static>(&self, callback: F) -> u32 {
        self.app.listen(Self::ADD_WALLPAPER_EVENT, move |event| {
            let data: AddWallpaper = serde_json::from_str(event.payload()).unwrap();

            callback(data);
        })
    }

    const REMOVE_WALLPAPER_EVENT: &str = "remove-wallpaper";

    pub fn emit_remove_wallpaper(&self, id: Uuid) -> anyhow::Result<()> {
        Ok(self.app.emit(Self::REMOVE_WALLPAPER_EVENT, id)?)
    }

    pub fn listen_remove_wallpaper<F: Fn(Uuid) + Send + 'static>(&self, callback: F) -> u32 {
        self.app.listen(Self::REMOVE_WALLPAPER_EVENT, move |event| {
            let id: Uuid = serde_json::from_str(event.payload()).unwrap();

            callback(id);
        })
    }
}

pub mod payload {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};

    use crate::config::{Filter, Wallpaper, WallpaperSource};

    /// Represents the payload for applying wallpaper settings.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApplyWallpaper {
        pub application_path: Option<PathBuf>,
        pub filters: Option<Vec<Filter>>,
        pub opacity: Option<f64>,
        pub source: Option<WallpaperSource>,
    }

    /// Represents the payload for adding a new wallpaper configuration.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddWallpaper {
        pub id: uuid::Uuid,
        pub wallpaper: Wallpaper,
    }
}

pub fn setup_event_manager(app: &tauri::App) {
    let manager = EventManager::new(app.handle().clone());
    state::set_event_manager_state(app.handle(), manager);
}

pub mod state {
    use tauri::Manager;

    use crate::event_manager::EventManager;

    pub type EventManagerState = EventManager;

    pub(super) fn set_event_manager_state(app: &tauri::AppHandle, manager: EventManager) {
        app.manage(manager);
    }
}
