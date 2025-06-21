use std::sync::Arc;

use window_getter::Window;
use window_observer::{tokio::sync::mpsc, Event, WindowObserver};

use super::{WallpaperConfig, WallpaperWindows};

pub struct OverlayManager {
    observer: WindowObserver,
}

impl OverlayManager {
    pub async fn start(wallpaper_windows: WallpaperWindows, app_pid: u32) -> Option<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        let observer = match WindowObserver::start(
            app_pid,
            tx,
            window_observer::smallvec![Event::Resized, Event::Moved, Event::Activated],
        )
        .await
        {
            Ok(observer) => observer,
            Err(window_observer::Error::InvalidProcessID(_)) => return None,
            Err(e) => panic!("Failed to start window observer: {e:?}"),
        };

        spawn_order_management_task(rx, wallpaper_windows);

        Some(Self { observer })
    }

    pub async fn stop(self) {
        self.observer
            .stop()
            .await
            .expect("Failed to stop window observer");
    }
}

fn spawn_order_management_task(
    mut rx: mpsc::UnboundedReceiver<(window_observer::Window, Event)>,
    wallpaper_windows: WallpaperWindows,
) {
    tauri::async_runtime::spawn(async move {
        while let Some((window, event)) = rx.recv().await {
            let window: Option<Window> = window.try_into().expect("Failed to convert window");

            if let Some(window) = window {
                tauri::async_runtime::spawn(manage_order(
                    event,
                    window,
                    Arc::clone(&wallpaper_windows),
                ));
            }
        }
    });
}

async fn manage_order(event: Event, window: Window, wallpaper_windows: WallpaperWindows) {
    let wallpaper_windows = wallpaper_windows.lock().await;
    let Some(wallpaper_window) = wallpaper_windows.get(&window.id()) else {
        return;
    };

    match event {
        Event::Moved => {
            let bounds = window.bounds().expect("Failed to get window bounds");
            wallpaper_window
                .set_position(tauri::LogicalPosition::new(bounds.x(), bounds.y()))
                .expect("Failed to set wallpaper window position");
        }
        Event::Resized => {
            let bounds = window.bounds().expect("Failed to get window bounds");
            wallpaper_window
                .set_size(tauri::LogicalSize::new(bounds.width(), bounds.height()))
                .expect("Failed to set wallpaper window size");
        }
        Event::Activated => {
            wallpaper_window
                .set_always_on_top(true)
                .expect("Failed to set always on top");
        }
        Event::Deactivated => {
            wallpaper_window
                .set_always_on_top(false)
                .expect("Failed to set always on top");
        }
        _ => {}
    }
}
