use window_getter::WindowId;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub trait WindowExt {
    fn set_opacity(&self, opacity: f64) -> anyhow::Result<()>;
    fn set_order_above(&self, relative_to: WindowId) -> anyhow::Result<()>;
}
