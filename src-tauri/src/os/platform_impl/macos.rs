use anyhow::Result;
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
    fn setup_platform_specific(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn set_opacity(&self, opacity: f64) -> Result<()> {
        let ns_window = get_ns_window(self);
        unsafe { ns_window.setAlphaValue(opacity) };

        Ok(())
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

pub mod custom_feature {
    use objc2_app_kit::NSWindowCollectionBehavior;
    use tauri::{Manager, WebviewWindow};

    pub fn setup_collection_behavior(window: WebviewWindow) {
        let app = window.app_handle().clone();

        app.run_on_main_thread(move || {
            let ns_window = super::get_ns_window(&window);

            unsafe {
                ns_window.setCollectionBehavior(
                    NSWindowCollectionBehavior::Auxiliary
                        | NSWindowCollectionBehavior::Transient
                        | NSWindowCollectionBehavior::CanJoinAllSpaces
                        | NSWindowCollectionBehavior::FullScreenAllowsTiling
                        | NSWindowCollectionBehavior::IgnoresCycle,
                );
            }
        })
        .unwrap();
    }
}

/// Implementations of private API Core Graphics Services bindings.
mod core_graphics_services {
    #![allow(non_upper_case_globals)]
    #![allow(dead_code)]

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
