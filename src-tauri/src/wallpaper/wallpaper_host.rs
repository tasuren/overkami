use std::sync::Arc;

use tauri::{AppHandle, async_runtime::Mutex};
use uuid::Uuid;

use crate::{
    commands::sync::ApplyWallpaper,
    config::Wallpaper,
    os::application_observer::{listen_application, unlisten_application},
    wallpaper::overlay_host::OverlayHost,
};

pub type OverlayHosts = Arc<Mutex<Vec<OverlayHost>>>;
pub type SharedWallpaperConfig = Arc<Mutex<Wallpaper>>;

/// The host of wallpaper for the application.
/// It tracks app launches and if the wallpaper application is need, apply it to the app.
pub struct WallpaperHost {
    id: Uuid,
    config: SharedWallpaperConfig,
    overlay_hosts: OverlayHosts,
}

impl WallpaperHost {
    pub async fn new(app: AppHandle, id: Uuid, config: Wallpaper) -> Self {
        log::info!("Create wallpaper host for ID: {id}");

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

        Self {
            id,
            config,
            overlay_hosts,
        }
    }

    /// Stop wallpaper.
    pub async fn stop(self) {
        let config = self.config.lock().await;
        unlisten_application(&config.application_name, self.id).await;

        for overlay in self.overlay_hosts.lock().await.drain(..) {
            overlay.stop().await;
        }
    }

    /// Apply new wallpaper settings.
    pub async fn apply_wallpaper(&self, old_wallpaper: Wallpaper, mut payload: ApplyWallpaper) {
        if let Some(new_app_name) = payload.application_name.take() {
            self.change_application(old_wallpaper.application_name, new_app_name)
                .await;
        }

        for overlay_host in self.overlay_hosts.lock().await.iter() {
            overlay_host
                .apply_wallpaper(payload.opacity, payload.source.clone())
                .await;
        }
    }

    /// Change the target application of wallpaper.
    async fn change_application(&self, old_app_name: String, new_app_name: String) {
        // When the application is updated, we need to clear the overlay hosts.
        // Then we will set up new overlay hosts with the updated configuration.
        let old_overlay_hosts = std::mem::take(&mut *self.overlay_hosts.lock().await);

        for overlay_host in old_overlay_hosts {
            overlay_host.stop().await;
        }

        // Register the new application listener due to application change.
        if let Some(tx) = unlisten_application(&old_app_name, self.id).await {
            listen_application(tx, new_app_name, self.id).await;
        };
    }
}

/// Handle application rise and fall events to make or stop wallpaper instances.
mod application_updates {
    use tauri::{AppHandle, async_runtime};
    use uuid::Uuid;

    use crate::{
        os::application_observer::{ApplicationEvent, listen_application},
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
            listen_application(tx, config.application_name.clone(), wallpaper_id).await;
        }

        async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                log::debug!(
                    "Received application event: \
                    event = {event:?}, \
                    wallpaper_id = {wallpaper_id}"
                );

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

                let overlay_host = OverlayHost::start(app.clone(), wallpaper_id, pid, &config)
                    .await
                    .unwrap();

                if let Some(overlay_host) = overlay_host {
                    overlay_hosts.lock().await.push(overlay_host);
                } else {
                    log::info!("Overlay host creation is skipped");
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
