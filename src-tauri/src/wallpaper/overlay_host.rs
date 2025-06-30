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
pub type FiltersState = Arc<Mutex<Vec<Filter>>>;

/// Manages overlay windows for a specific process ID.
///
/// This struct is responsible for observing window events
/// and managing overlays based on the provided filters.
pub struct OverlayHost {
    wallpaper_id: Uuid,
    pid: u32,
    observer: WindowObserver,
    overlays: Overlays,
    event_listener: u32,
    app: AppHandle,
}

impl OverlayHost {
    pub async fn start(
        wallpaper_id: Uuid,
        pid: u32,
        config: &Wallpaper,
        app: AppHandle,
    ) -> anyhow::Result<Option<Self>> {
        log::info!(
            "Starting new overlay host: wallpaper_id = {}, pid = {}",
            wallpaper_id,
            pid
        );

        let filters = Arc::new(Mutex::new(config.filters.clone()));

        // Initialize the observer and overlays.
        let (tx, rx) = mpsc::unbounded_channel();
        let Some(observer) = observer::start_observer(wallpaper_id, pid, tx)
            .await
            .context("Failed to start observer")?
        else {
            return Ok(None);
        };

        let overlays: Overlays = Default::default();
        observer::spawn_overlay_management_task(wallpaper_id, pid, Arc::clone(&overlays), rx);

        // Listen for configuration changes.
        let event_listener = app.state::<EventManager>().listen_apply_wallpaper({
            let filters = Arc::clone(&filters);

            move |data| {
                let filters = Arc::clone(&filters);

                async_runtime::spawn(async move {
                    if let Some(new) = data.filters {
                        log::info!(
                            "Updating filters for overlay host: \
                            wallpaper_id = {wallpaper_id}, \
                            pid = {pid}, \
                            filters = {new:?}"
                        );

                        let _ = std::mem::replace(&mut *filters.lock().await, new);
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
            .create_windows(&filters, &config.source, config.opacity)
            .await;

        Ok(Some(overlay_host))
    }

    async fn create_windows(&self, filters: &FiltersState, source: &WallpaperSource, opacity: f64) {
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
                    self.app.clone(),
                    Arc::clone(filters),
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
    use std::sync::Arc;

    use pollster::FutureExt;
    use uuid::Uuid;
    use window_getter::Window;
    use window_observer::{
        tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender},
        Event, WindowObserver,
    };

    use crate::wallpaper::overlay_host::Overlays;

    pub async fn start_observer(
        wallpaper_id: Uuid,
        pid: u32,
        tx: UnboundedSender<(window_observer::Window, Event)>,
    ) -> anyhow::Result<Option<WindowObserver>> {
        log::info!("Starting window observer: wallpaper_id = {wallpaper_id}, pid = {pid}");

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
                start_observer_with_retry(start, wallpaper_id, pid)
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
        wallpaper_id: Uuid,
        pid: u32,
    ) -> anyhow::Result<Option<WindowObserver>> {
        // On macOS, the application that has just been launched may not be ready
        // to observe yet. So we need to retry.

        let retry_timeout = std::time::Instant::now();

        loop {
            if retry_timeout.elapsed() > std::time::Duration::from_secs(5) {
                log::info!(
                    "Failed to start window observer because \
                    it may does not support accessibility API: \
                    wallpaper_id = {wallpaper_id}, pid = {pid}"
                );

                break Ok(None);
            }

            match start() {
                Err(window_observer::Error::InvalidProcessToObserve { .. }) => {
                    log::info!(
                        "Retrying to start window observer: \
                        wallpaper_id = {wallpaper_id}, pid = {pid}"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(100));

                    continue;
                }
                observer => break observer.map(Some).map_err(|e| e.into()),
            }
        }
    }

    pub fn spawn_overlay_management_task(
        wallpaper_id: Uuid,
        pid: u32,
        overlays: Overlays,
        mut rx: UnboundedReceiver<(window_observer::Window, Event)>,
    ) {
        tauri::async_runtime::spawn(async move {
            while let Some((target_window, event)) = rx.recv().await {
                let target_window: Option<Window> = target_window.try_into().ok().flatten();

                if let Some(target_window) = target_window {
                    log::info!(
                        "Received window event for window: \
                        target_window = {:?}, \
                        event = {event:?}, \
                        wallpaper_id = {wallpaper_id}, \
                        pid = {pid}",
                        target_window.id()
                    );

                    manage_overlay(Arc::clone(&overlays), event, target_window).await;
                } else {
                    log::error!(
                        "Received window event but window is invalid as \
                        `window_getter::Window`: \
                        event = {event:?}, \
                        wallpaper_id = {wallpaper_id}, \
                        pid = {pid}"
                    );
                }
            }
        });
    }

    async fn manage_overlay(overlays: Overlays, event: Event, window: Window) {
        let overlays = overlays.lock().await;
        let Some(overlay) = overlays.get(&window.id()) else {
            return;
        };

        overlay.handle_target_window_event(event, &window).await;
    }
}
