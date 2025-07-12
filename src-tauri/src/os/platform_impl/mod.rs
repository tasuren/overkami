use window_getter::WindowId;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub use macos::custom_feature;
#[cfg(target_os = "windows")]
#[allow(unused_imports)]
pub use windows::custom_feature;

pub trait WebviewWindowPlatformExt {
    fn setup_platform_specific(&self) -> anyhow::Result<()>;
    fn set_opacity(&self, opacity: f64) -> anyhow::Result<()>;
    fn set_order_above(&self, relative_to: WindowId) -> anyhow::Result<()>;
    fn merge_always_on_top(&self, top: bool) -> anyhow::Result<()>;
    fn merge_ignore_cursor_events(&self, ignore: bool) -> anyhow::Result<()>;
}

pub trait WindowPlatformExt {
    fn is_frontmost(&self) -> anyhow::Result<bool>;
}

pub fn activate_another_app(target: &window_getter::Window) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    return macos::application::activate_another_app(target);
    #[cfg(target_os = "windows")]
    return windows::set_foreground_window(target);
}
