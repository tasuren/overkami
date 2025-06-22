use std::{collections::HashMap, sync::Arc};

use tauri::{
    async_runtime::{self, Mutex},
    AppHandle, Listener, Manager,
};
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
    app: AppHandle,
    pid: u32,
    filters: FiltersState,
    observer: WindowObserver,
    overlays: Overlays,
    event_listener: u32,
}

impl OverlayHost {
    pub async fn start(
        app: AppHandle,
        pid: u32,
        config: &Wallpaper,
        windows: &[Window],
    ) -> Option<Self> {
        let filters = Arc::new(Mutex::new(config.filters.clone()));

        // Initialize the observer and overlays.
        let (tx, rx) = mpsc::unbounded_channel();
        let observer = observer::start_observer(tx, pid).await?;

        let overlays: Overlays = Default::default();
        observer::spawn_overlay_management_task(rx, Arc::clone(&overlays));

        // Listen for configuration changes.
        let event_listener = app.state::<EventManager>().listen_apply_wallpaper({
            let filters = Arc::clone(&filters);

            move |data| {
                let filters = Arc::clone(&filters);

                async_runtime::spawn(async move {
                    apply_config::apply_changes(&filters, data).await;
                });
            }
        });

        let overlay_host = Self {
            app: app.clone(),
            pid,
            filters,
            observer,
            overlays,
            event_listener,
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
                    app.clone(),
                    window.clone(),
                    &config.name,
                    &config.source,
                    config.opacity,
                );

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

mod apply_config {
    use super::FiltersState;
    use crate::event_manager::payload::ApplyWallpaper;

    pub async fn apply_changes(filters: &FiltersState, data: ApplyWallpaper) {
        if let Some(new) = data.filters {
            let _ = std::mem::replace(&mut *filters.lock().await, new);
        }
    }
}

mod observer {
    use std::sync::Arc;

    use pollster::FutureExt;
    use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
    use window_getter::Window;
    use window_observer::{Event, WindowObserver};

    use crate::wallpaper::overlay_host::Overlays;

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
        mut rx: UnboundedReceiver<(window_observer::Window, Event)>,
        overlays: Overlays,
    ) {
        tauri::async_runtime::spawn(async move {
            while let Some((window, event)) = rx.recv().await {
                let window: Option<Window> = window.try_into().expect("Failed to convert window");

                if let Some(window) = window {
                    manage_overlay(event, window, Arc::clone(&overlays)).await;
                }
            }
        });
    }

    async fn manage_overlay(event: Event, window: Window, overlays: Overlays) {
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
}
