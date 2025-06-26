use std::sync::Arc;

use tauri::{async_runtime::Mutex, AppHandle, Listener, Manager};

use crate::{
    config::Wallpaper, os::application_observer::unlisten_application,
    wallpaper::overlay_host::OverlayHost, EventManagerState,
};

pub type OverlayHosts = Arc<Mutex<Vec<OverlayHost>>>;
pub type SharedWallpaperConfig = Arc<Mutex<Wallpaper>>;

pub struct WallpaperHost {
    app: AppHandle,
    config: SharedWallpaperConfig,
    event_listener: u32,
}

impl WallpaperHost {
    pub async fn new(app: AppHandle, config: Wallpaper) -> Self {
        let event_manager_state = app.state::<EventManagerState>();
        let overlay_hosts: OverlayHosts = Default::default();
        let config = Arc::new(Mutex::new(config));

        // Listen for application process changes and set up overlay hosts.
        application_updates::setup_event_listener(
            app.clone(),
            Arc::clone(&overlay_hosts),
            Arc::clone(&config),
        )
        .await;

        // Listen for configuration updates and apply them.
        let event_listener = config_updates::setup_event_listener(
            &event_manager_state,
            Arc::clone(&config),
            overlay_hosts,
        );

        Self {
            app,
            config,
            event_listener,
        }
    }

    pub async fn stop(self) {
        let config = self.config.lock().await;

        unlisten_application(&config.application.path, config.id).await;
        self.app.unlisten(self.event_listener);
    }
}

/// Handle application rise and fall events.
mod application_updates {
    use tauri::{async_runtime, AppHandle};

    use crate::{
        os::{
            application_observer::{listen_application, ApplicationEvent},
            windows::get_windows,
        },
        wallpaper::{overlay_host::OverlayHost, wallpaper_host::SharedWallpaperConfig},
    };

    use super::OverlayHosts;

    pub async fn setup_event_listener(
        app: tauri::AppHandle,
        overlay_hosts: OverlayHosts,
        config: SharedWallpaperConfig,
    ) {
        let (tx, mut rx) = async_runtime::channel(100);

        {
            let config = config.lock().await;
            listen_application(tx, config.application.path.clone(), config.id).await;
        }

        async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                on_application_event(&app, &overlay_hosts, &config, event).await;
            }
        });
    }

    async fn on_application_event(
        app: &AppHandle,
        overlay_hosts: &OverlayHosts,
        config: &SharedWallpaperConfig,
        event: ApplicationEvent,
    ) {
        match event {
            ApplicationEvent::Added(pid) => {
                let config = config.lock().await;
                let windows = get_windows().await;

                let overlay_host = OverlayHost::start(app.clone(), pid, &config, &windows).await;

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

    use super::{OverlayHosts, SharedWallpaperConfig};
    use crate::{
        event_manager::{payload::ApplyWallpaper, EventManager},
        os::application_observer::{listen_application, unlisten_application},
    };

    pub fn setup_event_listener(
        event_manager: &EventManager,
        config: SharedWallpaperConfig,
        overlay_hosts: OverlayHosts,
    ) -> u32 {
        event_manager.listen_apply_wallpaper(move |payload| {
            let overlay_hosts = Arc::clone(&overlay_hosts);
            let config = Arc::clone(&config);

            tauri::async_runtime::spawn(async move {
                apply_wallpaper(config, overlay_hosts, payload).await;
            });
        })
    }

    pub async fn apply_wallpaper(
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
            if let Some(tx) = unlisten_application(&old_app.path, config.id).await {
                listen_application(tx, config.application.path.clone(), config.id).await;
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
