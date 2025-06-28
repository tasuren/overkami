use std::{collections::HashMap, sync::Arc};

use tauri::{
    async_runtime::{self, Mutex},
    AppHandle, Listener, Manager,
};
use uuid::Uuid;
use window_getter::{Window, WindowId};
use window_observer::{tokio::sync::mpsc, WindowObserver};

use crate::{
    config::{Filter, Wallpaper},
    event_manager::EventManager,
    wallpaper::overlay::Overlay,
};

pub type Overlays = Arc<Mutex<HashMap<WindowId, Overlay>>>;
pub type FiltersState = Arc<Mutex<Vec<Filter>>>;

/// Manages overlay windows for a specific process ID.
///
/// This struct is responsible for observing window events
/// and managing overlays based on the provided filters.
pub struct OverlayHost {
    _wallpaper_id: Uuid,
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
        windows: &[Window],
        app: AppHandle,
    ) -> Option<Self> {
        let filters = Arc::new(Mutex::new(config.filters.clone()));

        // Initialize the observer and overlays.
        let (tx, rx) = mpsc::unbounded_channel();
        let observer = observer::start_observer(tx, pid).await?;

        let overlays: Overlays = Default::default();
        observer::spawn_overlay_management_task(Arc::clone(&filters), Arc::clone(&overlays), rx);

        // Listen for configuration changes.
        let event_listener = app.state::<EventManager>().listen_apply_wallpaper({
            let filters = Arc::clone(&filters);

            move |data| {
                let filters = Arc::clone(&filters);

                async_runtime::spawn(async move {
                    if let Some(new) = data.filters {
                        let _ = std::mem::replace(&mut *filters.lock().await, new);
                    }
                });
            }
        });

        let overlay_host = Self {
            _wallpaper_id: wallpaper_id,
            pid,
            observer,
            overlays,
            event_listener,
            app: app.clone(),
        };

        // Initialize overlays for existing windows.
        let mut overlays = overlay_host.overlays.lock().await;

        for window in windows {
            if let Ok(window_pid) = window.owner_pid() {
                if window_pid as u32 != pid {
                    continue;
                }

                let window_id = window.id();
                let overlay = Overlay::new(
                    wallpaper_id,
                    window.clone(),
                    &config.source,
                    config.opacity,
                    app.clone(),
                )
                .await;

                overlays.insert(window_id, overlay);
            }
        }

        drop(overlays);
        Some(overlay_host)
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub async fn stop(self) {
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
    use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
    use window_getter::Window;
    use window_observer::{Event, WindowObserver};

    use crate::wallpaper::overlay_host::{FiltersState, Overlays};

    pub async fn start_observer(
        tx: UnboundedSender<(window_observer::Window, Event)>,
        pid: u32,
    ) -> Option<WindowObserver> {
        // `WindowObserver::start` future will not be `Send`, so we need to
        // spawn it on a blocking thread. Otherwise, we can't spawn tasks
        // that use this `start_observer` in the async runtime.

        let window_observer = tauri::async_runtime::spawn_blocking(move || {
            WindowObserver::start(
                pid,
                tx,
                window_observer::smallvec![
                    Event::Resized,
                    Event::Moved,
                    Event::Activated,
                    Event::Deactivated
                ],
            )
            .block_on()
        })
        .await
        .expect("Failed to start window observer");

        match window_observer {
            Ok(observer) => Some(observer),
            Err(window_observer::Error::InvalidProcessID(_)) => None,
            Err(e) => panic!("Failed to start window observer: {e:?}"),
        }
    }

    pub fn spawn_overlay_management_task(
        filters: FiltersState,
        overlays: Overlays,
        mut rx: UnboundedReceiver<(window_observer::Window, Event)>,
    ) {
        tauri::async_runtime::spawn(async move {
            while let Some((window, event)) = rx.recv().await {
                let window: Option<Window> = window.try_into().expect("Failed to convert window");

                if let Some(window) = window {
                    manage_overlay(Arc::clone(&filters), Arc::clone(&overlays), event, window)
                        .await;
                }
            }
        });
    }

    async fn manage_overlay(
        filters: FiltersState,
        overlays: Overlays,
        event: Event,
        window: Window,
    ) {
        if let Ok(title) = window.title() {
            if !super::filter::wallpaper_filter(title, &filters.lock().await) {
                return;
            };
        } else {
            return;
        }

        let overlays = overlays.lock().await;
        let Some(overlay) = overlays.get(&window.id()) else {
            return;
        };

        match event {
            Event::Moved => overlay.on_move(&window),
            Event::Resized => overlay.on_resize(&window),
            Event::Activated => overlay.on_activate(),
            Event::Deactivated => overlay.on_deactivate(window.id()).await,
            _ => {}
        }
    }
}

mod filter {
    use crate::config::{Filter, StringFilterStrategy};

    pub fn string_filter(
        target: impl AsRef<str>,
        search: impl AsRef<str>,
        strategy: &StringFilterStrategy,
    ) -> bool {
        let target = target.as_ref();
        let search = search.as_ref();

        match strategy {
            StringFilterStrategy::Prefix => target.starts_with(search),
            StringFilterStrategy::Suffix => target.ends_with(search),
            StringFilterStrategy::Contains => target.contains(search),
            StringFilterStrategy::Exact => target == search,
        }
    }

    pub fn wallpaper_filter(window_name: Option<String>, filters: &[Filter]) -> bool {
        filters.iter().all(|filter| match (&window_name, filter) {
            (Some(window_name), Filter::WindowName { name, strategy }) => {
                string_filter(window_name, name, strategy)
            }
            _ => false,
        })
    }
}
