#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod macos;

pub trait WindowExt {
    fn set_opacity(&self, opacity: f64);
}
