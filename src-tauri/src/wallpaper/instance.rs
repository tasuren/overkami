use std::{collections::HashMap, sync::Arc};

use tauri::{async_runtime::Mutex, AppHandle, WebviewWindow, WebviewWindowBuilder};
use window_getter::{Window, WindowId};

use super::OverlayManager;
use crate::{
    config::Wallpaper,
    os::{ApplicationProcess, WindowExt},
};

pub type WallpaperWindows = Arc<Mutex<HashMap<WindowId, WebviewWindow>>>;
pub type WallpaperConfig = Arc<Mutex<Wallpaper>>;

pub struct WallpaperInstance {
    app: AppHandle,
    config: WallpaperConfig,
    windows: WallpaperWindows,
    overlay: Option<OverlayManager>,
}

impl WallpaperInstance {
    pub fn new(app: AppHandle, config: Wallpaper) -> Self {
        let windows = WallpaperWindows::default();

        Self {
            app,
            config: Arc::new(Mutex::new(config)),
            windows,
            overlay: None,
        }
    }

    pub fn config(&self) -> WallpaperConfig {
        Arc::clone(&self.config)
    }

    pub async fn add_wallpaper_windows(&mut self, target_windows: &[Window]) {
        let mut windows = self.windows.lock().await;
        let config = self.config.lock().await;

        for window in target_windows {
            let window_id = window.id();
            let Some(bounds) = window.bounds().ok() else {
                continue;
            };

            let window = WebviewWindowBuilder::new(
                &self.app,
                format!("wallpaper-{}-{}", config.name, windows.len()),
                settings::get_wallpaper_url(&config.source),
            )
            .decorations(false)
            .position(bounds.x(), bounds.y())
            .inner_size(bounds.width(), bounds.height())
            .transparent(true)
            .build()
            .expect("WebViewのウィンドウの作成に失敗しました。");

            window.set_opacity(config.opacity);
            window
                .set_always_on_top(true)
                .expect("Failed to set always on top");
            window
                .set_ignore_cursor_events(true)
                .expect("Failed to set ignore cursor events");
            println!("aa");

            windows.insert(window_id, window);
        }
    }

    /// Start the wallpaper instance.
    /// It should be called when the target application is started.
    pub async fn start(&mut self, application: ApplicationProcess) {
        let app_pid = application.pid as i32;
        let windows = tauri::async_runtime::spawn_blocking(move || {
            window_getter::get_windows()
                .expect("Failed to get windows")
                .into_iter()
                .filter(|window| window.owner_pid().is_ok_and(|pid| pid == app_pid))
                .collect::<Vec<window_getter::Window>>()
        })
        .await
        .expect("Failed to get windows");

        if !windows.is_empty() {
            self.add_wallpaper_windows(&windows).await;
        }

        // Create overlay manager for tracing window position change.
        let Some(overlay) = OverlayManager::start(Arc::clone(&self.windows), application.pid).await
        else {
            // If the overlay manager returns `None`, it means that pid is not valid.
            self.stop().await;

            return;
        };
        println!("ccc");
        self.overlay = Some(overlay)
    }

    /// Stop the wallpaper instance.
    /// It should be called when the target application is closed.
    pub async fn stop(&mut self) {
        if let Some(overlay) = self.overlay.take() {
            overlay.stop().await;
        };

        let windows = std::mem::replace(&mut *self.windows.lock().await, HashMap::new());

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
