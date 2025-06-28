use std::sync::Arc;

use tauri::{async_runtime::Mutex, AppHandle, Listener, Manager};
use uuid::Uuid;

use crate::{
    config::Wallpaper, os::application_observer::unlisten_application,
    wallpaper::overlay_host::OverlayHost, EventManagerState,
};

pub type OverlayHosts = Arc<Mutex<Vec<OverlayHost>>>;
pub type SharedWallpaperConfig = Arc<Mutex<Wallpaper>>;

pub struct WallpaperHost {
    id: Uuid,
    config: SharedWallpaperConfig,
    app: AppHandle,
    event_listener: u32,
}

impl WallpaperHost {
    pub async fn new(id: Uuid, config: Wallpaper, app: AppHandle) -> Self {
        let event_manager_state = app.state::<EventManagerState>();
        let overlay_hosts: OverlayHosts = Default::default();
        let config = Arc::new(Mutex::new(config));

        // Listen for application process changes and set up overlay hosts.
        application_updates::setup_event_listener(
            id,
            Arc::clone(&config),
            Arc::clone(&overlay_hosts),
            app.clone(),
        )
        .await;

        // Listen for configuration updates and apply them.
        let event_listener = config_updates::setup_event_listener(
            id,
            Arc::clone(&config),
            overlay_hosts,
            &event_manager_state,
        );

        Self {
            id,
            config,
            app,
            event_listener,
        }
    }

    pub async fn stop(self) {
        let config = self.config.lock().await;

        unlisten_application(&config.application.path, self.id).await;
        self.app.unlisten(self.event_listener);
    }
}

/// Handle application rise and fall events.
mod application_updates {
    use tauri::{async_runtime, AppHandle};
    use uuid::Uuid;

    use crate::{
        os::{
            application_observer::{listen_application, ApplicationEvent},
            windows::get_windows,
        },
        wallpaper::{overlay_host::OverlayHost, wallpaper_host::SharedWallpaperConfig},
    };

    use super::OverlayHosts;

    pub async fn setup_event_listener(
        wallpaper_id: uuid::Uuid,
        config: SharedWallpaperConfig,
        overlay_hosts: OverlayHosts,
        app: tauri::AppHandle,
    ) {
        let (tx, mut rx) = async_runtime::channel(100);

        {
            let config = config.lock().await;
            listen_application(tx, config.application.path.clone(), wallpaper_id).await;
        }

        async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                on_application_event(wallpaper_id, &config, event, &overlay_hosts, &app).await;
            }
        });
    }

    async fn on_application_event(
        wallpaper_id: Uuid,
        config: &SharedWallpaperConfig,
        event: ApplicationEvent,
        overlay_hosts: &OverlayHosts,
        app: &AppHandle,
    ) {
        match event {
            ApplicationEvent::Added(pid) => {
                let config = config.lock().await;
                let windows = get_windows().await;

                let overlay_host =
                    OverlayHost::start(wallpaper_id, pid, &config, &windows, app.clone()).await;

                if let Some(overlay_host) = overlay_host {
                    overlay_hosts.lock().await.push(overlay_host);
                }
            }
            ApplicationEvent::Removed(pid) => {
                let mut overlay_hosts = overlay_hosts.lock().await;
                let index = overlay_hosts.iter().position(|host| host.pid() == pid);

                if let Some(index) = index {
                    let overlay_host = overlay_hosts.remove(index);
                    overlay_host.stop().await;
                };
            }
        }
    }
}

/// Handles configuration updates for the wallpaper host.
mod config_updates {
    use std::sync::Arc;

    use uuid::Uuid;

    use super::{OverlayHosts, SharedWallpaperConfig};
    use crate::{
        event_manager::{payload::ApplyWallpaper, EventManager},
        os::application_observer::{listen_application, unlisten_application},
    };

    pub fn setup_event_listener(
        wallpaper_id: Uuid,
        config: SharedWallpaperConfig,
        overlay_hosts: OverlayHosts,
        event_manager: &EventManager,
    ) -> u32 {
        event_manager.listen_apply_wallpaper(move |payload| {
            let overlay_hosts = Arc::clone(&overlay_hosts);
            let config = Arc::clone(&config);

            tauri::async_runtime::spawn(async move {
                apply_wallpaper(wallpaper_id, config, overlay_hosts, payload).await;
            });
        })
    }

    pub async fn apply_wallpaper(
        wallpaper_id: Uuid,
        config: SharedWallpaperConfig,
        overlay_hosts: OverlayHosts,
        payload: ApplyWallpaper,
    ) {
        let mut config = config.lock().await;

        if let Some(application) = payload.application {
            // If the application path has changed, we need to update the application listener.

            let old_app = std::mem::replace(&mut config.application, application);

            // When the application is updated, we need to clear the overlay hosts.
            // Then we will set up new overlay hosts with the updated configuration.
            let old_overlay_hosts = std::mem::take(&mut *overlay_hosts.lock().await);

            for overlay_host in old_overlay_hosts {
                overlay_host.stop().await;
            }

            // Register the new application listener due to application change.
            if let Some(tx) = unlisten_application(&old_app.path, wallpaper_id).await {
                listen_application(tx, config.application.path.clone(), wallpaper_id).await;
            };
        }

        if let Some(filters) = payload.filters {
            config.filters = filters;
        }

        if let Some(opacity) = payload.opacity {
            config.opacity = opacity;
        }

        if let Some(source) = payload.source {
            config.source = source;
        }
    }
}
