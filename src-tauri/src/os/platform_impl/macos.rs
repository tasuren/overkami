use anyhow::{Context as _, Result};
use objc2::rc::Retained;
use objc2_app_kit::NSWindow;
use tauri::WebviewWindow;
use window_getter::WindowId;

pub fn get_ns_window(window: &WebviewWindow) -> Retained<NSWindow> {
    let ptr = window.ns_window().expect("Failed to get NSWindow");

    // SAFETY: We are assuming that the pointer is valid and correctly typed.
    // It is derived from a Tauri `WebviewWindow`, which is expected to be an `NSWindow`.
    unsafe { Retained::retain_autoreleased(ptr as *mut NSWindow).unwrap() }
}

impl super::WindowExt for WebviewWindow {
    fn set_opacity(&self, opacity: f64) {
        let ns_window = get_ns_window(self);
        unsafe { ns_window.setAlphaValue(opacity) };
    }

    fn set_order_above(&self, relative_to: WindowId) -> Result<()> {
        let ns_window = get_ns_window(self);
        let window_id = unsafe { ns_window.windowNumber() };

        let result = core_graphics_services::cgs_order_window(
            core_graphics_services::cgs_default_connection_for_thread(),
            window_id as _,
            core_graphics_services::kCGSOrderAbove,
            *relative_to.inner(),
        );

        if let Err(error) = result {
            anyhow::bail!("Failed to set window order above: {:?}", error);
        }

        Ok(())
    }
}

mod core_graphics_services {
    use std::ffi::c_int;

    use objc2_core_graphics::{CGError, CGWindowID};

    pub type CGSConnectionID = c_int;

    pub type CGSWindowOrderingMode = c_int;
    pub const kCGSOrderBelow: CGSWindowOrderingMode = -1;
    pub const kCGSOrderOut: CGSWindowOrderingMode = 0;
    pub const kCGSOrderAbove: CGSWindowOrderingMode = 1;
    pub const kCGSOrderIn: CGSWindowOrderingMode = 2;

    extern "C" {
        fn CGSMainConnectionID() -> CGSConnectionID;
        fn CGSDefaultConnectionForThread() -> CGSConnectionID;
        fn CGSOrderWindow(
            cid: CGSConnectionID,
            wid: CGWindowID,
            mode: CGSWindowOrderingMode,
            relative_to_wid: CGWindowID,
        ) -> CGError;
    }

    pub fn cgs_default_connection_for_thread() -> CGSConnectionID {
        unsafe { CGSDefaultConnectionForThread() }
    }

    pub fn cgs_main_connection_id() -> CGSConnectionID {
        unsafe { CGSMainConnectionID() }
    }

    pub fn cgs_order_window(
        connection_id: CGSConnectionID,
        window: CGWindowID,
        mode: CGSWindowOrderingMode,
        relative_to_window: CGWindowID,
    ) -> Result<(), CGError> {
        let result = unsafe { CGSOrderWindow(connection_id, window, mode, relative_to_window) };

        if result == CGError::Success {
            Ok(())
        } else {
            Err(result)
        }
    }
}
