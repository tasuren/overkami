pub use filter::*;
pub use instance::*;

mod instance {
    use tauri::{AppHandle, WebviewWindow, WebviewWindowBuilder};

    use super::filter::wallpaper_filter;
    use crate::model::config;

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

        pub fn get_config(&mut self) -> &config::WallpaperSource {
            return &self.config.source;
        }

        pub fn add_wallpaper_window(&mut self) {
            let window = WebviewWindowBuilder::new(
                &self.app,
                format!("wallpaper-{}-{}", &self.config.name, self.windows.len()),
                super::settings::get_wallpaper_url(&self.config.source),
            )
            .build()
            .expect("WebViewのウィンドウの作成に失敗しました。");

            self.windows.push(window);
        }

        pub fn on_app_create(&mut self, app_name: String) {
            if !wallpaper_filter(Some(app_name), None, &self.config.filters) {
                return;
            }

            self.add_wallpaper_window();
        }

        pub fn on_window_create(&mut self, app_name: String, window_name: String) {
            if !wallpaper_filter(Some(app_name), Some(window_name), &self.config.filters) {
                return;
            };

            self.add_wallpaper_window();
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

    pub fn wallpaper_filter(
        app_name: Option<String>,
        window_name: Option<String>,
        filters: &Vec<Filter>,
    ) -> bool {
        filters
            .iter()
            .all(|filter| match (&app_name, &window_name, filter) {
                (_, Some(window_name), Filter::WindowName { name, strategy }) => {
                    string_filter(window_name, name, strategy)
                }
                _ => false,
            })
    }
}

pub mod settings {
    use tauri::{Url, WebviewUrl};

    use crate::{model::config::WallpaperSource, utils::convert_file_src};

    /// ビルトインの壁紙を使う際に必要なデータを用意する。
    /// これはHTMLを指定する形式の壁紙には対応していない。それはカスタム壁紙であり、ビルトイン壁紙ではない。
    pub fn get_wallpaper_url(source: &WallpaperSource) -> WebviewUrl {
        match source {
            WallpaperSource::Picture { .. } => {
                WebviewUrl::App("/builtin-wallpapers/picture.html".into())
            }
            WallpaperSource::Video { .. } => {
                WebviewUrl::App("/builtin-wallpapers/video.html".into())
            }
            WallpaperSource::LocalWebPage { path } => {
                WebviewUrl::External(Url::parse(&convert_file_src(path).unwrap()).unwrap())
            }
            WallpaperSource::RemoteWebPage { url } => {
                WebviewUrl::External(Url::parse(url).unwrap())
            }
        }
    }
}
