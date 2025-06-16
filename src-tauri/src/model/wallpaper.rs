pub use filter::*;
pub use instance::*;

mod instance {
    use tauri::{AppHandle, WebviewWindow, WebviewWindowBuilder};
    use window_getter::Window;

    use super::filter::wallpaper_filter;
    use crate::{
        model::{
            config::{self, Application},
            wallpaper,
        },
        os::{ApplicationProcess, WindowExt},
    };

    pub struct WallpaperInstance {
        app: AppHandle,
        config: config::Wallpaper,
        windows: Vec<WebviewWindow>,
    }

    impl WallpaperInstance {
        pub fn new(app: AppHandle, config: config::Wallpaper) -> Self {
            Self {
                app,
                config,
                windows: Vec::new(),
            }
        }

        pub fn config(&mut self) -> &config::WallpaperSource {
            &self.config.source
        }

        pub async fn add_wallpaper_windows(&mut self, target_windows: &[Window]) {
            for window in target_windows {
                let Some(bounds) = window.bounds().ok() else {
                    continue;
                };

                let window = WebviewWindowBuilder::new(
                    &self.app,
                    format!("wallpaper-{}-{}", &self.config.name, self.windows.len()),
                    super::settings::get_wallpaper_url(&self.config.source),
                )
                .decorations(false)
                .position(bounds.x(), bounds.y())
                .inner_size(bounds.width(), bounds.height())
                .transparent(true)
                .build()
                .expect("WebViewのウィンドウの作成に失敗しました。");

                window.set_opacity(self.config.opacity);
                window
                    .set_always_on_top(true)
                    .expect("Failed to set always on top");
                window
                    .set_ignore_cursor_events(true)
                    .expect("Failed to set ignore cursor events");

                self.windows.push(window);
            }
        }

        pub async fn on_new_app(&mut self, application: ApplicationProcess) {
            if application.path != self.config.application.path {
                return;
            }

            // Get current windows owned by the application.
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

            self.add_wallpaper_windows(&windows).await;
        }

        pub fn on_window_event(
            &mut self,
            window: window_observer::Window,
            event: window_observer::Event,
        ) {
            if !wallpaper_filter(window.get_title().ok(), &self.config.filters) {
                return;
            };
        }
    }
}

mod filter {
    use crate::model::config::{Filter, StringFilterStrategy};

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

    use crate::{model::config::WallpaperSource, utils::convert_file_src};

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
