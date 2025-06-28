use tauri::{AppHandle, Listener, Manager, WebviewWindow, WebviewWindowBuilder};
use uuid::Uuid;
use window_getter::{Window, WindowId};

use crate::{
    config::WallpaperSource,
    event_manager::payload::ApplyWallpaper,
    os::WindowExt,
    utils::{adjust_position, adjust_size},
    EventManagerState,
};

/// Represents an overlay window for a wallpaper.
///
/// This struct has responsibility for managing overlay window.
/// e.g. position, size, opacity, source, and opacity or source configuration updates.
pub struct Overlay {
    window: WebviewWindow,
    listener: u32,
}

impl Overlay {
    pub async fn new(
        wallpaper_id: Uuid,
        target_window: Window,
        source: &WallpaperSource,
        opacity: f64,
        app: AppHandle,
    ) -> Self {
        let window = create_window(&app, &wallpaper_id, &target_window, source, opacity);

        // Listen for updates of config
        let listener = app.state::<EventManagerState>().listen_apply_wallpaper({
            let window = window.clone();

            move |payload| {
                on_apply_wallpaper(&window, payload);
            }
        });

        let overlay = Self { window, listener };

        // Set initial position and size
        overlay.on_move(&target_window);
        overlay.on_resize(&target_window);

        // Set overlay order.
        overlay.on_order_change(target_window.id()).await;

        overlay
    }

    pub fn on_move(&self, target_window: &Window) {
        let bounds = target_window
            .bounds()
            .expect("Failed to get target window bounds");
        let position = adjust_position(&self.window, bounds.x(), bounds.y());

        self.window
            .set_position(position)
            .expect("Failed to set wallpaper window position");
    }

    pub fn on_resize(&self, target_window: &Window) {
        let bounds = target_window
            .bounds()
            .expect("Failed to get target window bounds");
        let size = adjust_size(&self.window, bounds.width(), bounds.height());

        self.window
            .set_size(size)
            .expect("Failed to set wallpaper window size");
    }

    pub fn on_activate(&self) {
        self.window
            .set_always_on_top(true)
            .expect("Failed to set always on top");
    }

    pub async fn on_deactivate(&self, target_window_id: WindowId) {
        self.window
            .set_always_on_top(false)
            .expect("Failed to set always on top");

        self.on_order_change(target_window_id).await;
    }

    pub async fn on_order_change(&self, target_window_id: WindowId) {
        #[cfg(target_os = "macos")]
        {
            // On macOS, we can't set the order above immediately.
            // So we need to wait a bit.
            // TODO: Find a better way to handle this.

            self.window
                .set_order_above(target_window_id)
                .expect("Failed to set order above");

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        self.window
            .set_order_above(target_window_id)
            .expect("Failed to set order above");
    }

    pub fn close(&self) {
        self.window.app_handle().unlisten(self.listener);

        self.window
            .close()
            .expect("Failed to close wallpaper window");
    }
}

pub fn create_window(
    app: &AppHandle,
    wallpaper_id: &Uuid,
    target_window: &Window,
    source: &WallpaperSource,
    opacity: f64,
) -> WebviewWindow {
    let window = WebviewWindowBuilder::new(
        app,
        format!("wallpaper-{}-{}", wallpaper_id, target_window.id().as_u32()),
        source::get_wallpaper_url(source),
    )
    .decorations(false)
    .resizable(false)
    .transparent(true)
    .skip_taskbar(true)
    .focused(false)
    .build()
    .expect("Failed to create wallpaper window");

    window.set_ignore_cursor_events(true).unwrap();
    window
        .setup_platform_specific()
        .expect("Failed to setup platform specific settings");
    window.set_opacity(opacity).unwrap();

    #[cfg(target_os = "macos")]
    {
        use crate::os::platform_impl::macos::custom_feature;

        custom_feature::setup_collection_behavior(window.clone());
    }

    window
}

pub fn on_apply_wallpaper(window: &WebviewWindow, payload: ApplyWallpaper) {
    if let Some(opacity) = payload.opacity {
        window
            .set_opacity(opacity)
            .expect("Failed to set wallpaper window opacity");
    }

    if let Some(source) = payload.source {
        let url = source::get_wallpaper_url(&source);

        window
            .eval(format!("window.location.replace('{url}');"))
            .expect("Failed to update wallpaper window URL");
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
