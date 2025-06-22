use tauri::{AppHandle, Emitter, Listener};

pub struct EventManager {
    app: AppHandle,
}

impl EventManager {
    fn new(app: AppHandle) -> Self {
        Self { app }
    }

    const APPLY_WALLPAPER_EVENT: &str = "apply-wallpaper";

    pub fn emit_apply_wallpaper(&self, data: payload::ApplyWallpaper) {
        self.app
            .emit(Self::APPLY_WALLPAPER_EVENT, data)
            .expect("Failed to emit apply-wallpaper event");
    }

    pub fn listen_apply_wallpaper<F: Fn(payload::ApplyWallpaper) + Send + 'static>(
        &self,
        callback: F,
    ) -> u32 {
        self.app.listen(Self::APPLY_WALLPAPER_EVENT, move |event| {
            let data: payload::ApplyWallpaper = serde_json::from_str(event.payload())
                .expect("Failed to deserialize apply-wallpaper payload from event");

            callback(data);
        })
    }
}

pub mod payload {
    use serde::{Deserialize, Serialize};

    use crate::config::{Application, Filter, WallpaperSource};

    /// Represents the payload for applying wallpaper settings.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApplyWallpaper {
        pub application: Option<Application>,
        pub filters: Option<Vec<Filter>>,
        pub opacity: Option<f64>,
        pub source: Option<WallpaperSource>,
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
