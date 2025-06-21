mod application_monitor;
pub mod platform_impl;
mod window_observer;

pub use application_monitor::{ApplicationMonitor, ApplicationProcess};
pub use platform_impl::WindowExt;
