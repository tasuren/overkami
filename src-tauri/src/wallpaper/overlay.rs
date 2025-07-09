use tauri::{AppHandle, WebviewWindow, WebviewWindowBuilder};
use uuid::Uuid;
use window_getter::Window;
use window_observer::Event;

use crate::{
    config::{Filter, WallpaperSource},
    os::{platform_impl::WindowPlatformExt, WebviewWindowPlatformExt},
    utils::{adjust_position, adjust_size},
};

/// Represents an overlay window for a wallpaper.
///
/// This struct has responsibility for managing overlay window.
/// e.g. position, size, opacity, source, and opacity or source configuration updates.
pub struct Overlay {
    wallpaper_id: Uuid,
    target_window: Window,
    overlay_window: WebviewWindow,
}

impl Overlay {
    /// Check if the overlay should be created for the given target window.
    pub async fn should_handle(target_window: &Window, filters: &[Filter]) -> bool {
        match target_window.title() {
            Ok(title) => {
                if !filter::wallpaper_filter(title, filters) {
                    return false;
                }
            }
            Err(e) => {
                log::info!(
                    "Failed to get title for {:?}, ignoring this window. Detail: {e}",
                    target_window.id()
                );

                return false;
            }
        }

        let bounds = match target_window.bounds() {
            Ok(bounds) => bounds,
            Err(e) => {
                log::info!(
                    "Failed to get window bounds for {:?}, ignoreing this window. Detail: {e}",
                    target_window.id()
                );

                return false;
            }
        };

        // On windows, some windows may have zero width or height.
        if bounds.width() == 0. || bounds.height() == 0. {
            log::info!(
                "{:?} has no width or height, ignoreing this window.",
                target_window.id()
            );

            return false;
        }

        true
    }

    pub async fn new(
        wallpaper_id: Uuid,
        target_window: Window,
        source: &WallpaperSource,
        opacity: f64,
        filters: &[Filter],
        app: AppHandle,
    ) -> Option<Self> {
        if !Self::should_handle(&target_window, filters).await {
            return None;
        }

        log::info!(
            "Create new overlay: wallpaper_id = {wallpaper_id}, target_window_id = {:?}",
            target_window.id()
        );

        let overlay_window = create_window(&app, &wallpaper_id, &target_window, source, opacity);

        // Listen for updates of config
        let overlay = Self {
            wallpaper_id,
            target_window,
            overlay_window,
        };

        overlay.setup_initial_window_state().await;

        Some(overlay)
    }

    async fn setup_initial_window_state(&self) {
        // Set initial overlay order.
        match self.target_window.is_frontmost() {
            Err(e) => log::warn!(
                "Failed to check if window {:?} is frontmost, \
                skipping always on top. Detail: {e}",
                self.target_window.id()
            ),
            Ok(true) => self.activate().await,
            Ok(false) => self.deactivate().await,
        }

        // Set initial position and size
        self.move_(self.target_window.bounds().unwrap());
        self.resize(self.target_window.bounds().unwrap());
    }

    pub fn apply_wallpaper(&self, opacity: Option<f64>, source: Option<WallpaperSource>) {
        if let Some(opacity) = opacity {
            log::info!("Update wallpaper overlay opacity to {opacity}");

            self.overlay_window.set_opacity(opacity).unwrap();
        }

        if let Some(source) = source {
            log::info!("Update wallpaper overlay opacity to {source:?}");

            let url = source::get_wallpaper_url(&source);

            self.overlay_window
                .eval(format!("window.location.replace('{url}');"))
                .unwrap();
        }
    }

    pub async fn handle_target_window_event(&self, event: Event, target_window: &Window) {
        match event {
            Event::Moved => self.move_(target_window.bounds().unwrap()),
            Event::Resized => self.resize(target_window.bounds().unwrap()),
            Event::Activated => self.activate().await,
            Event::Deactivated => self.deactivate().await,
            _ => {}
        }
    }

    pub fn move_(&self, bounds: window_getter::Bounds) {
        let position = adjust_position(
            self.overlay_window.scale_factor().unwrap(),
            bounds.x(),
            bounds.y(),
        );

        self.overlay_window.set_position(position).unwrap();
    }

    pub fn resize(&self, bounds: window_getter::Bounds) {
        let size = adjust_size(
            self.overlay_window.scale_factor().unwrap(),
            bounds.width(),
            bounds.height(),
        );

        self.overlay_window.set_size(size).unwrap();
    }

    pub async fn activate(&self) {
        self.set_order().await;
        self.overlay_window.set_always_on_top(true).unwrap();
    }

    pub async fn deactivate(&self) {
        self.overlay_window.set_always_on_top(false).unwrap();

        self.set_order().await;
    }

    pub async fn set_order(&self) {
        #[cfg(target_os = "macos")]
        {
            // On macOS, we can't set the order above immediately.
            // So we need to wait a bit.
            // TODO: Find a better way to handle this problem.

            self.overlay_window
                .set_order_above(self.target_window.id())
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        self.overlay_window
            .set_order_above(self.target_window.id())
            .unwrap();
    }

    pub fn close(&self) {
        log::info!(
            "Closing overlay window: wallpaper_id = {}, target_window_id = {:?}",
            self.wallpaper_id,
            self.target_window.id()
        );

        self.overlay_window.close().unwrap();
    }
}

pub fn create_window(
    app: &AppHandle,
    wallpaper_id: &Uuid,
    target_window: &Window,
    source: &WallpaperSource,
    opacity: f64,
) -> WebviewWindow {
    let label = format!("wallpaper-{}-{}", wallpaper_id, target_window.id().as_u32());
    log::info!("Create overlay window with label `{label}`.");

    let window = WebviewWindowBuilder::new(app, label, source::get_wallpaper_url(source))
        .decorations(false)
        .resizable(false)
        .transparent(true)
        .skip_taskbar(true)
        .focused(false)
        .build()
        .unwrap();

    window.set_ignore_cursor_events(true).unwrap();
    window.setup_platform_specific().unwrap();
    window.set_opacity(opacity).unwrap();

    #[cfg(target_os = "macos")]
    {
        use crate::os::platform_impl::custom_feature;

        custom_feature::setup_collection_behavior(window.clone());
    }

    window
}

pub mod source {
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
                    location.to_str().expect("Failed to read video location"),
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
