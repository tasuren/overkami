use std::sync::Arc;

use tauri::{async_runtime::Mutex, AppHandle, Manager};

use crate::{
    config::Wallpaper,
    os::application_observer::unlisten_application,
    wallpaper::{
        overlay_host::OverlayHost, wallpaper_host::application_updates::setup_application_updates,
    },
    EventManagerState,
};

pub type OverlayHosts = Arc<Mutex<Vec<OverlayHost>>>;
pub type SharedWallpaperConfig = Arc<Mutex<Wallpaper>>;

pub struct WallpaperHost {
    app: AppHandle,
    config: SharedWallpaperConfig,
    overlay_hosts: OverlayHosts,
    event_listener: u32,
}

impl WallpaperHost {
    pub async fn new(app: AppHandle, config: Wallpaper) -> Self {
        let event_manager_state = app.state::<EventManagerState>();
        let overlay_hosts: OverlayHosts = Default::default();
        let config = Arc::new(Mutex::new(config));

        // Listen for application process changes and set up overlay hosts.
        setup_application_updates(app.clone(), Arc::clone(&overlay_hosts), Arc::clone(&config))
            .await;

        // Listen for configuration updates and apply them.
        let event_listener = config_updates::setup_event_listener(
            app.clone(),
            &event_manager_state,
            Arc::clone(&config),
            Arc::clone(&overlay_hosts),
        );

        Self {
            app,
            config,
            overlay_hosts,
            event_listener,
        }
    }

    pub fn config(&self) -> &SharedWallpaperConfig {
        &self.config
    }

    pub async fn setup_initial_windows(&self, pid: u32, windows: &[window_getter::Window]) {
        let config = self.config.lock().await;

        if let Some(overlay_host) =
            OverlayHost::start(self.app.clone(), pid, &config, &windows).await
        {
            self.overlay_hosts.lock().await.push(overlay_host);
        }
    }

    pub async fn stop(self) {
        let config = self.config.lock().await;

        unlisten_application(&config.application.path, config.id).await;
    }
}

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

    pub async fn setup_application_updates(
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

    pub async fn on_application_event(
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

mod config_updates {
    use std::sync::Arc;

    use pollster::FutureExt;
    use tauri::AppHandle;

    use super::{manage_overlay_hosts::setup_overlay_hosts, OverlayHosts, SharedWallpaperConfig};
    use crate::event_manager::{payload::ApplyWallpaper, EventManager};

    pub fn setup_event_listener(
        app: AppHandle,
        event_manager: &EventManager,
        config: SharedWallpaperConfig,
        overlay_hosts: OverlayHosts,
    ) -> u32 {
        event_manager.listen_apply_wallpaper(move |payload| {
            let overlay_hosts = Arc::clone(&overlay_hosts);
            let config = Arc::clone(&config);
            let app = app.clone();

            tauri::async_runtime::spawn(async move {
                apply_wallpaper(&app, config, overlay_hosts, payload).await;
            });
        })
    }

    pub async fn apply_wallpaper(
        app: &AppHandle,
        config: SharedWallpaperConfig,
        overlay_hosts: OverlayHosts,
        payload: ApplyWallpaper,
    ) {
        if let Some(application) = payload.application {
            config.lock().await.application = application;

            // When the application is updated, we need to clear the overlay hosts.
            // Then we will set up new overlay hosts with the updated configuration.
            reset_overlay_hosts(app.clone(), overlay_hosts, &config).await;
        }

        let mut config = config.lock().await;

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

    async fn reset_overlay_hosts(
        app: AppHandle,
        overlay_hosts: OverlayHosts,
        config: &SharedWallpaperConfig,
    ) {
        let config = Arc::clone(config);
        let old_overlay_hosts = std::mem::take(&mut *overlay_hosts.lock().await);

        for overlay_host in old_overlay_hosts {
            overlay_host.stop().block_on();
        }

        let config = config.lock().await;
        let new_overlay_hosts = setup_overlay_hosts(&app, &config).await;

        *overlay_hosts.lock().await = new_overlay_hosts;
    }
}

mod manage_overlay_hosts {
    use std::collections::HashSet;

    use tauri::AppHandle;

    use crate::{
        config::Wallpaper,
        os::{
            application_monitor::{get_application_processes, ApplicationProcess},
            windows::get_windows,
        },
        wallpaper::overlay_host::OverlayHost,
    };

    pub async fn setup_overlay_hosts(app: &AppHandle, config: &Wallpaper) -> Vec<OverlayHost> {
        let processes: Vec<ApplicationProcess> =
            get_application_processes(|path| config.application.path == path);

        let mut overlay_hosts = Vec::new();
        let pids = processes.into_iter().map(|p| p.pid).collect::<HashSet<_>>();
        let windows = get_windows().await;

        for pid in pids {
            if let Some(overlay_host) = OverlayHost::start(app.clone(), pid, config, &windows).await
            {
                overlay_hosts.push(overlay_host);
            };
        }

        overlay_hosts
    }
}
