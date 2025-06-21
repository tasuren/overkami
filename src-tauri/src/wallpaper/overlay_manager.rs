use std::sync::Arc;

use tauri::AppHandle;
use window_getter::Window;
use window_observer::{tokio::sync::mpsc, Event, WindowObserver};

use crate::{
    os::WindowExt,
    utils::{adjust_position, adjust_size},
};

use super::WallpaperWindows;

pub struct OverlayManager {
    observer: WindowObserver,
    pid: u32,
}

impl OverlayManager {
    pub async fn start(
        app: AppHandle,
        wallpaper_windows: WallpaperWindows,
        pid: u32,
    ) -> Option<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        println!("start actual observer");
        let observer = match WindowObserver::start(
            pid,
            tx,
            window_observer::smallvec![
                Event::Resized,
                Event::Moved,
                Event::Activated,
                Event::Deactivated
            ],
        )
        .await
        {
            Ok(observer) => observer,
            Err(window_observer::Error::InvalidProcessID(_)) => return None,
            Err(e) => panic!("Failed to start window observer: {e:?}"),
        };

        spawn_order_management_task(app, rx, wallpaper_windows);

        Some(Self { observer, pid })
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub async fn stop(self) {
        self.observer
            .stop()
            .await
            .expect("Failed to stop window observer");
    }
}

fn spawn_order_management_task(
    app: AppHandle,
    mut rx: mpsc::UnboundedReceiver<(window_observer::Window, Event)>,
    wallpaper_windows: WallpaperWindows,
) {
    println!("spawn observer task");
    tauri::async_runtime::spawn(async move {
        println!("aaa");
        while let Some((window, event)) = rx.recv().await {
            let window: Option<Window> = window.try_into().expect("Failed to convert window");
            println!("{event:?}");

            if let Some(window) = window {
                tauri::async_runtime::spawn(manage_order(
                    app.clone(),
                    event,
                    window,
                    Arc::clone(&wallpaper_windows),
                ));
            }
        }
    });
}

async fn manage_order(
    app: AppHandle,
    event: Event,
    window: Window,
    wallpaper_windows: WallpaperWindows,
) {
    let wallpaper_windows = wallpaper_windows.lock().await;
    println!("{event:?}");
    let Some(wallpaper_window) = wallpaper_windows.get(&window.id()) else {
        return;
    };

    println!("aaa");
    match event {
        Event::Moved => {
            let bounds = window.bounds().expect("Failed to get window bounds");
            wallpaper_window
                .set_position(adjust_position(wallpaper_window, bounds.x(), bounds.y()))
                .expect("Failed to set wallpaper window position");
        }
        Event::Resized => {
            let bounds = window.bounds().expect("Failed to get window bounds");
            wallpaper_window
                .set_size(adjust_size(
                    wallpaper_window,
                    bounds.width(),
                    bounds.height(),
                ))
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

            #[cfg(target_os = "macos")]
            {
                wallpaper_window
                    .set_order_above(window.id())
                    .expect("Failed to set order above");

                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }

            wallpaper_window
                .set_order_above(window.id())
                .expect("Failed to set order above");
        }
        _ => {}
    }
}
