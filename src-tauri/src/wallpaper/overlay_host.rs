use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use tauri::{
    async_runtime::{self, Mutex},
    AppHandle, Listener, Manager,
};
use uuid::Uuid;
use window_getter::WindowId;
use window_observer::{tokio::sync::mpsc, WindowObserver};

use crate::{
    config::{Filter, Wallpaper, WallpaperSource},
    event_manager::EventManager,
    os::windows::get_windows,
    wallpaper::overlay::Overlay,
};

pub type Overlays = Arc<Mutex<HashMap<WindowId, Overlay>>>;

/// Manages overlay windows for a specific process ID.
///
/// This struct is responsible for observing window events
/// and managing overlays based on the provided filters.
pub struct OverlayHost {
    app: AppHandle,
    wallpaper_id: Uuid,
    pid: u32,
    observer: WindowObserver,
    overlays: Overlays,
    event_listener: u32,
}

impl OverlayHost {
    pub async fn start(
        app: AppHandle,
        wallpaper_id: Uuid,
        pid: u32,
        config: &Wallpaper,
    ) -> anyhow::Result<Option<Self>> {
        log::info!(
            "Start new overlay host: wallpaper_id = {}, pid = {}",
            wallpaper_id,
            pid
        );

        // Initialize the observer and overlays.
        let (tx, rx) = mpsc::unbounded_channel();
        let Some(observer) = observer::start_observer(pid, tx)
            .await
            .context("Failed to start observer")?
        else {
            return Ok(None);
        };

        let overlays: Overlays = Default::default();
        overlay_management::spawn_overlay_management_task(
            app.clone(),
            wallpaper_id,
            pid,
            Arc::clone(&overlays),
            rx,
        );

        // Listen for configuration changes.
        let event_listener = app.state::<EventManager>().listen_apply_wallpaper({
            move |payload| {
                if payload.id != wallpaper_id {
                    return;
                }

                async_runtime::spawn(async move {
                    if let Some(new) = payload.filters {
                        log::info!(
                            "Updating filters for overlay host: \
                            wallpaper_id = {wallpaper_id}, \
                            pid = {pid}, \
                            filters = {new:?}"
                        );
                    }
                });
            }
        });

        let overlay_host = Self {
            wallpaper_id,
            pid,
            observer,
            overlays,
            event_listener,
            app: app.clone(),
        };

        // Initialize overlays for existing windows.
        overlay_host
            .create_windows(&config.filters, &config.source, config.opacity)
            .await;

        Ok(Some(overlay_host))
    }

    async fn create_windows(&self, filters: &[Filter], source: &WallpaperSource, opacity: f64) {
        let mut overlays = self.overlays.lock().await;

        for window in get_windows().await {
            if let Ok(window_pid) = window.owner_pid() {
                if window_pid as u32 != self.pid {
                    continue;
                }

                let window_id = window.id();
                let Some(overlay) = Overlay::new(
                    self.wallpaper_id,
                    window.clone(),
                    source,
                    opacity,
                    filters,
                    self.app.clone(),
                )
                .await
                else {
                    continue;
                };

                overlays.insert(window_id, overlay);
            }
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub async fn stop(self) {
        log::info!(
            "Stopping overlay host: wallpaper_id = {}, pid = {}",
            self.wallpaper_id,
            self.pid
        );

        self.observer
            .stop()
            .await
            .expect("Failed to stop window observer");
        self.app.unlisten(self.event_listener);

        for overlay in self.overlays.lock().await.values() {
            overlay.close();
        }
    }
}

mod observer {
    use pollster::FutureExt;
    use window_observer::{tokio::sync::mpsc::UnboundedSender, Event, WindowObserver};

    pub async fn start_observer(
        pid: u32,
        tx: UnboundedSender<(window_observer::Window, Event)>,
    ) -> anyhow::Result<Option<WindowObserver>> {
        log::info!("Starting window observer for PID {pid}.");

        // `WindowObserver::start` future will not be `Send`, so we need to
        // spawn it on a blocking thread. Otherwise, we can't spawn tasks
        // that use this `start_observer` in the async runtime.

        tauri::async_runtime::spawn_blocking(move || {
            let start = move || {
                WindowObserver::start(
                    pid,
                    tx.clone(),
                    window_observer::smallvec![
                        Event::Resized,
                        Event::Moved,
                        Event::Activated,
                        Event::Deactivated
                    ],
                )
                .block_on()
            };

            #[cfg(target_os = "macos")]
            {
                start_observer_with_retry(start)
            }
            #[cfg(target_os = "windows")]
            {
                start().map(Some).map_err(|e| e.into())
            }
        })
        .await
        .unwrap()
    }

    #[cfg(target_os = "macos")]
    fn start_observer_with_retry(
        start: impl Fn() -> Result<WindowObserver, window_observer::Error>,
    ) -> anyhow::Result<Option<WindowObserver>> {
        // On macOS, the application that has just been launched may not be ready
        // to observe yet. So we need to retry.

        let retry_timeout = std::time::Instant::now();

        loop {
            if retry_timeout.elapsed() > std::time::Duration::from_secs(30) {
                log::warn!(
                    "Failed to start window observer because \
                    it may does not support accessibility API or some reason."
                );

                break Ok(None);
            }

            match start() {
                Err(window_observer::Error::InvalidProcessId(_)) => {
                    log::debug!("Retrying to start window observer...");
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    continue;
                }
                Err(window_observer::Error::NotSupported) => {
                    log::info!(
                        "Failed to start window observer because \
                        it does not support accessibility API."
                    );

                    // TODO: Return an error to frontend.

                    break Ok(None);
                }
                observer => break observer.map(Some).map_err(|e| e.into()),
            }
        }
    }
}

mod overlay_management {
    use std::sync::Arc;

    use tauri::{AppHandle, Manager};
    use tokio::sync::mpsc::UnboundedReceiver;
    use uuid::Uuid;
    use window_getter::Window;
    use window_observer::Event;

    use crate::{wallpaper::overlay::Overlay, ConfigState};

    use super::Overlays;

    pub fn spawn_overlay_management_task(
        app: AppHandle,
        wallpaper_id: Uuid,
        pid: u32,
        overlays: Overlays,
        mut rx: UnboundedReceiver<(window_observer::Window, Event)>,
    ) {
        tauri::async_runtime::spawn(async move {
            while let Some((target_window, event)) = rx.recv().await {
                log::debug!(
                    "Received window event {event:?}: \
                    wallpaper_id = {wallpaper_id}, \
                    pid = {pid}"
                );

                let target_window: Option<Window> = target_window.try_into().ok().flatten();

                if let Some(target_window) = target_window {
                    manage_overlay(
                        wallpaper_id,
                        event,
                        target_window,
                        Arc::clone(&overlays),
                        app.clone(),
                    )
                    .await;
                    continue;
                } else {
                    log::error!("Received window event but it is invalid as window_getter.");
                }
            }
        });
    }

    async fn manage_overlay(
        wallpaper_id: Uuid,
        event: Event,
        window: Window,
        overlays: Overlays,
        app: AppHandle,
    ) {
        let mut overlays = overlays.lock().await;

        let Some(overlay) = overlays.get(&window.id()) else {
            log::debug!(
                "New window is detected, creating new overlay for it: {:?}",
                window.id()
            );

            let config = app.state::<ConfigState>();
            let config = config.lock().await;
            let Some(wallpaper) = config.wallpapers.get(&wallpaper_id) else {
                log::warn!("No wallpaper found so skip overlay creation.");

                return;
            };

            let window_id = window.id();
            let overlay = Overlay::new(
                wallpaper_id,
                window,
                &wallpaper.source,
                wallpaper.opacity,
                &wallpaper.filters,
                app.clone(),
            )
            .await;

            if let Some(overlay) = overlay {
                overlays.insert(window_id, overlay);
            }

            return;
        };

        overlay.handle_target_window_event(event, &window).await;
    }
}
