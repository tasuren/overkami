use tauri::{AppHandle, Listener, Manager, WebviewWindow, WebviewWindowBuilder};
use uuid::Uuid;
use window_getter::Window;
use window_observer::Event;

use crate::{
    config::{Filter, WallpaperSource},
    event_manager::payload::ApplyWallpaper,
    os::{platform_impl::WindowPlatformExt, WebviewWindowPlatformExt},
    utils::{adjust_position, adjust_size},
    wallpaper::overlay_host::FiltersState,
    EventManagerState,
};

/// Represents an overlay window for a wallpaper.
///
/// This struct has responsibility for managing overlay window.
/// e.g. position, size, opacity, source, and opacity or source configuration updates.
pub struct Overlay {
    target_window: Window,
    overlay_window: WebviewWindow,
    listener: u32,
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
                    "Failed to get title for {:?}, ignoring it. Detail: {e}",
                    target_window.id()
                );

                return false;
            }
        }

        let bounds = match target_window.bounds() {
            Ok(bounds) => bounds,
            Err(e) => {
                log::info!(
                    "Failed to get window bounds for {:?}, ignoreing it. Detail: {e}",
                    target_window.id()
                );

                return false;
            }
        };

        // On windows, some windows may have zero width or height.
        if bounds.width() == 0. || bounds.height() == 0. {
            log::info!(
                "{:?} has no width or height, no overlay will be created.",
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
        app: AppHandle,
        filters: FiltersState,
    ) -> Option<Self> {
        if !Self::should_handle(&target_window, &filters.lock().await).await {
            return None;
        }

        log::info!(
            "Creating overlay for wallpaper ID {wallpaper_id} and target window {:?}",
            target_window.id()
        );

        let overlay_window = create_window(&app, &wallpaper_id, &target_window, source, opacity);

        // Listen for updates of config
        let listener = app.state::<EventManagerState>().listen_apply_wallpaper({
            let overlay_window = overlay_window.clone();

            move |payload| {
                on_apply_wallpaper(&overlay_window, payload);
            }
        });

        let overlay = Self {
            target_window,
            overlay_window,
            listener,
        };

        // Set initial overlay order.
        match overlay.target_window.is_frontmost() {
            Err(e) => log::info!(
                "Failed to check if window {:?} is frontmost, skipping always on top. Detail: {e}",
                overlay.target_window.id()
            ),
            Ok(true) => overlay.on_activate(),
            Ok(false) => overlay.on_deactivate().await,
        }

        // Set initial position and size
        overlay.on_move(overlay.target_window.bounds().unwrap());
        overlay.on_resize(overlay.target_window.bounds().unwrap());

        Some(overlay)
    }

    pub async fn handle_target_window_event(&self, event: Event, target_window: &Window) {
        log::info!(
            "Handling event {event:?} for target window {:?} on overlay {:?}",
            target_window.id(),
            self.overlay_window.label()
        );

        match event {
            Event::Moved => self.on_move(target_window.bounds().unwrap()),
            Event::Resized => self.on_resize(target_window.bounds().unwrap()),
            Event::Activated => self.on_activate(),
            Event::Deactivated => self.on_deactivate().await,
            _ => {}
        }
    }

    pub fn on_move(&self, bounds: window_getter::Bounds) {
        let position = adjust_position(&self.overlay_window, bounds.x(), bounds.y());

        self.overlay_window.set_position(position).unwrap();
    }

    pub fn on_resize(&self, bounds: window_getter::Bounds) {
        let size = adjust_size(&self.overlay_window, bounds.width(), bounds.height());

        self.overlay_window.set_size(size).unwrap();
    }

    pub fn on_activate(&self) {
        self.overlay_window.set_always_on_top(true).unwrap();
    }

    pub async fn on_deactivate(&self) {
        self.overlay_window.set_always_on_top(false).unwrap();

        self.on_order_change().await;
    }

    pub async fn on_order_change(&self) {
        #[cfg(target_os = "macos")]
        {
            // On macOS, we can't set the order above immediately.
            // So we need to wait a bit.
            // TODO: Find a better way to handle this problem.

            self.overlay_window
                .set_order_above(self.target_window.id())
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        self.overlay_window
            .set_order_above(self.target_window.id())
            .unwrap();
    }

    pub fn close(&self) {
        self.overlay_window.app_handle().unlisten(self.listener);

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
    log::info!("Creating overlay window with label: {label}");

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

pub fn on_apply_wallpaper(window: &WebviewWindow, payload: ApplyWallpaper) {
    if let Some(opacity) = payload.opacity {
        log::info!("Update wallpaper overlay opacity to {opacity}");

        window.set_opacity(opacity).unwrap();
    }

    if let Some(source) = payload.source {
        log::info!("Update wallpaper overlay opacity to {source:?}");

        let url = source::get_wallpaper_url(&source);

        window
            .eval(format!("window.location.replace('{url}');"))
            .unwrap();
    }
}

mod source {
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
