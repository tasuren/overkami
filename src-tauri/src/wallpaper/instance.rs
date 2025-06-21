use std::{collections::HashMap, sync::Arc};

use smallvec::SmallVec;
use tauri::{async_runtime::Mutex, AppHandle, WebviewWindow, WebviewWindowBuilder};
use window_getter::{Window, WindowId};

use super::OverlayManager;
use crate::{
    config::Wallpaper,
    os::WindowExt,
    utils::{adjust_position, adjust_size},
};

pub type WallpaperWindows = Arc<Mutex<HashMap<WindowId, WebviewWindow>>>;
pub type WallpaperConfig = Arc<Mutex<Wallpaper>>;

pub struct WallpaperInstance {
    app: AppHandle,
    config: WallpaperConfig,
    windows: WallpaperWindows,
    overlays: SmallVec<[OverlayManager; 5]>,
}

impl WallpaperInstance {
    pub fn new(app: AppHandle, config: Wallpaper) -> Self {
        let windows = WallpaperWindows::default();

        Self {
            app,
            config: Arc::new(Mutex::new(config)),
            windows,
            overlays: SmallVec::new(),
        }
    }

    pub fn config(&self) -> WallpaperConfig {
        Arc::clone(&self.config)
    }

    pub async fn add_wallpaper_windows(&mut self, target_windows: &[Window]) {
        let mut windows = self.windows.lock().await;
        let config = self.config.lock().await;

        for target_window in target_windows {
            let Some(bounds) = target_window.bounds().ok() else {
                continue;
            };

            // On windows, some windows have zero width and height.
            // These windows don't need be set wallpaper.
            if bounds.width() == 0. || bounds.height() == 0. {
                continue;
            }

            let window = WebviewWindowBuilder::new(
                &self.app,
                format!("wallpaper-{}-{}", config.name, target_window.id().as_u32()),
                settings::get_wallpaper_url(&config.source),
            )
            .decorations(false)
            .resizable(false)
            .transparent(true)
            .skip_taskbar(true)
            .build()
            .expect("Failed to create wallpaper window");

            let position = adjust_position(&window, bounds.x(), bounds.y());
            let size = adjust_size(&window, bounds.width(), bounds.height());

            window
                .set_position(position)
                .expect("Failed to set window position");
            window.set_size(size).expect("Failed to set window size");

            window.set_opacity(config.opacity).unwrap();
            window
                .set_ignore_cursor_events(true)
                .expect("Failed to set ignore cursor events");

            windows.insert(target_window.id(), window);
        }
    }

    /// Start the wallpaper instance.
    /// It should be called when the target application is started.
    pub async fn start(&mut self, pids: SmallVec<[u32; 5]>) {
        let (windows, pids) = tauri::async_runtime::spawn_blocking(move || {
            let windows = window_getter::get_windows()
                .expect("Failed to get windows")
                .into_iter()
                .filter(|window| {
                    window
                        .owner_pid()
                        .is_ok_and(|pid| pids.contains(&(pid as _)))
                })
                .collect::<Vec<window_getter::Window>>();

            (windows, pids)
        })
        .await
        .expect("Failed to spawn windows getter");

        if !windows.is_empty() {
            self.add_wallpaper_windows(&windows).await;
        }

        // Create overlay manager for tracing window position change.
        for pid in pids {
            let Some(overlay) =
                OverlayManager::start(self.app.clone(), Arc::clone(&self.windows), pid as _).await
            else {
                // If the overlay manager returns `None`, it means that pid is not valid.
                continue;
            };

            self.overlays.push(overlay);
        }
    }

    /// Stop the wallpaper instance.
    /// It should be called when the target application is closed.
    pub async fn stop(&mut self, pid: u32) {
        for (i, overlay) in self.overlays.iter().enumerate() {
            if overlay.pid() == pid {
                let overlay = self.overlays.remove(i);
                overlay.stop().await;

                break;
            }
        }
    }

    pub async fn stop_all(&mut self) {
        let overlays = std::mem::take(&mut self.overlays);

        for overlay in overlays.into_iter() {
            overlay.stop().await;
        }

        let windows = std::mem::take(&mut *self.windows.lock().await);

        for window in windows.into_values() {
            window
                .destroy()
                .expect("Failed to destroy wallpaper window");
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

pub mod settings {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    use tauri::{Url, WebviewUrl};

    use crate::{config::WallpaperSource, utils::convert_file_src};

    /// ビルトインの壁紙を使う際に必要なデータを用意する。
    /// これはHTMLを指定する形式の壁紙には対応していない。それはカスタム壁紙であり、ビルトイン壁紙ではない。
    pub fn get_wallpaper_url(source: &WallpaperSource) -> WebviewUrl {
        match source {
            WallpaperSource::Picture { location } => {
                let path = utf8_percent_encode(
                    location.to_str().expect("Failed to read picture location"),
                    NON_ALPHANUMERIC,
                );

                WebviewUrl::App(format!("?wallpaper=picture&path={path}").into())
            }
            WallpaperSource::Video { location } => {
                let path = utf8_percent_encode(
                    location.to_str().expect("Failed to read picture location"),
                    NON_ALPHANUMERIC,
                );

                WebviewUrl::App(format!("?wallpaper=video&path={path}").into())
            }
            WallpaperSource::LocalWebPage { location } => {
                WebviewUrl::External(Url::parse(&convert_file_src(location).unwrap()).unwrap())
            }
            WallpaperSource::RemoteWebPage { location } => {
                WebviewUrl::External(Url::parse(location).unwrap())
            }
        }
    }
}
