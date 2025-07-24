use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use tauri::{async_runtime::Mutex, AppHandle};
use uuid::Uuid;
use window_getter::WindowId;
use window_observer::{tokio::sync::mpsc, WindowObserver};

use crate::{
    config::{Filter, Wallpaper, WallpaperSource},
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
}

impl OverlayHost {
    pub async fn start(
        app: AppHandle,
        wallpaper_id: Uuid,
        pid: u32,
        config: &Wallpaper,
    ) -> anyhow::Result<Option<Self>> {
        log::info!(
            "Start new overlay host: \
            wallpaper_id = {wallpaper_id}, pid = {pid}"
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

        let overlay_host = Self {
            wallpaper_id,
            pid,
            observer,
            overlays,
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

        for overlay in self.overlays.lock().await.values() {
            overlay.close();
        }
    }

    pub async fn apply_wallpaper(&self, opacity: Option<f64>, source: Option<WallpaperSource>) {
        for overlay in self.overlays.lock().await.values() {
            overlay.apply_wallpaper(opacity, source.clone());
        }
    }
}

mod observer {
    use pollster::FutureExt;
    use window_observer::WindowObserver;

    pub async fn start_observer(
        pid: u32,
        tx: window_observer::EventTx,
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
                    window_observer::EventFilter {
                        foregrounded: true,
                        backgrounded: true,
                        moved: true,
                        resized: true,
                        closed: true,
                        hidden: true,
                        showed: true,
                        ..Default::default()
                    },
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
    use uuid::Uuid;
    use window_getter::{Window, WindowId};
    use window_observer::{Event, MaybeWindowAvailable};

    use crate::{wallpaper::overlay::Overlay, ConfigState};

    use super::Overlays;

    pub fn spawn_overlay_management_task(
        app: AppHandle,
        wallpaper_id: Uuid,
        pid: u32,
        overlays: Overlays,
        mut rx: window_observer::EventRx,
    ) {
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                let Ok(event) = event else {
                    log::warn!("Received invalid window event: {event:?}");
                    continue;
                };

                log::debug!(
                    "Received window event: \
                    event = {event:?}, \
                    wallpaper_id = {wallpaper_id}, \
                    pid = {pid}"
                );

                manage_overlay(app.clone(), wallpaper_id, event, Arc::clone(&overlays)).await;
            }
        });
    }

    async fn manage_overlay(
        app: AppHandle,
        wallpaper_id: Uuid,
        event: MaybeWindowAvailable,
        overlays: Overlays,
    ) {
        match event {
            MaybeWindowAvailable::Available { window, event } => match event {
                Event::Created => {
                    if let Some(window) = window.create_window_getter_window().ok().flatten() {
                        handle_window_created(app, wallpaper_id, window, overlays).await;
                    };
                }
                event => handle_general_event(app, wallpaper_id, window, event, overlays).await,
            },
            MaybeWindowAvailable::NotAvailable { event } => {
                if let Event::Closed { window_id } = event {
                    handle_window_closed(window_id, overlays).await;
                }
            }
        }
    }

    /// Handles window created events by creating a new overlay.
    async fn handle_window_created(
        app: AppHandle,
        wallpaper_id: Uuid,
        window: Window,
        overlays: Overlays,
    ) {
        log::debug!("New window is detected: {:?}", window.id());
        let mut overlays = overlays.lock().await;

        let config = app.state::<ConfigState>();
        let config = config.lock().await;
        let Some(wallpaper) = config.wallpapers.get(&wallpaper_id) else {
            log::warn!(
                "No wallpaper found so skip overlay creation for {:?}.",
                window.id()
            );

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
    }

    /// Handles window closed events by removing the overlay.
    async fn handle_window_closed(window_id: WindowId, overlays: Overlays) {
        log::debug!("Handling general window closed event: {window_id:?}");
        let mut overlays = overlays.lock().await;

        if let Some(overlay) = overlays.remove(&window_id) {
            overlay.close();
        }
    }

    /// Handles general window events that overlays might need to do action for overlay window.
    async fn handle_general_event(
        app: AppHandle,
        wallpaper_id: Uuid,
        window: window_observer::Window,
        event: Event,
        overlays: Overlays,
    ) {
        let Ok(window_id) = window.id() else {
            return;
        };
        log::debug!(
            "Handling general window event: \
            window_id = {window_id:?}, event = {event:?}",
        );

        {
            let mut overlays = overlays.lock().await;
            if let Some(overlay) = overlays.get_mut(&window_id) {
                overlay
                    .handle_target_window_event(window, event.clone())
                    .await;
                return;
            }
        }

        // If no overlay exists for the window, create a new one.
        if let Some(window) = window.create_window_getter_window().ok().flatten() {
            handle_window_created(app, wallpaper_id, window, overlays).await;
        }
    }
}
