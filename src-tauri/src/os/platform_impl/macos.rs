use objc2::rc::Retained;
use objc2_app_kit::NSWindow;
use tauri::WebviewWindow;

pub fn get_ns_window(window: &WebviewWindow) -> Retained<NSWindow> {
    let ptr = window.ns_window().expect("Failed to get NSWindow");

    // SAFETY: We are assuming that the pointer is valid and correctly typed.
    // It is derived from a Tauri `WebviewWindow`, which is expected to be an `NSWindow`.
    unsafe { Retained::retain_autoreleased(ptr as *mut NSWindow).unwrap() }
}

mod webview_window {
    use anyhow::{Context, Result};
    use tauri::{Manager, WebviewWindow};
    use window_getter::WindowId;

    use super::core_graphics_services;

    impl crate::os::WebviewWindowPlatformExt for WebviewWindow {
        fn setup_platform_specific(&self) -> anyhow::Result<()> {
            Ok(())
        }

        fn set_opacity(&self, opacity: f64) -> Result<()> {
            let app = self.app_handle().clone();
            let window = self.clone();

            app.run_on_main_thread(move || {
                let ns_window = super::get_ns_window(&window);
                unsafe { ns_window.setAlphaValue(opacity) };
            })
            .context("Failed to start opacity setting on main thread")?;

            Ok(())
        }

        fn set_order_above(&self, relative_to: WindowId) -> Result<()> {
            let ns_window = super::get_ns_window(self);
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

        fn merge_always_on_top(&self, top: bool) -> anyhow::Result<()> {
            Ok(self.set_always_on_top(top)?)
        }

        fn merge_ignore_cursor_events(&self, ignore: bool) -> anyhow::Result<()> {
            Ok(self.set_ignore_cursor_events(ignore)?)
        }
    }
}

mod window {
    use anyhow::Context as _;

    impl crate::os::WindowPlatformExt for window_getter::Window {
        fn is_frontmost(&self) -> anyhow::Result<bool> {
            let app =
                unsafe { objc2_app_kit::NSWorkspace::sharedWorkspace().frontmostApplication() };
            let Some(app) = app else { return Ok(false) };

            let pid = self.owner_pid().context("Failed to get owner PID")?;
            Ok(pid == unsafe { app.processIdentifier() })
        }
    }
}

pub mod application {
    use anyhow::Context as _;
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
    use window_getter::Window;

    pub fn activate_another_app(target: &Window) -> anyhow::Result<()> {
        let pid = target.owner_pid().context("Failed to get owner PID")?;
        let app =
            unsafe { NSRunningApplication::runningApplicationWithProcessIdentifier(pid as _) }
                .context("There are no running application with the given window")?;

        let result = unsafe { app.activateWithOptions(NSApplicationActivationOptions::empty()) };
        if !result {
            anyhow::bail!("Something went wrong while activating the application");
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

    pub fn set_document_edited(window: WebviewWindow, edited: bool) {
        let app = window.app_handle().clone();

        app.run_on_main_thread(move || {
            let ns_window = super::get_ns_window(&window);
            ns_window.setDocumentEdited(edited);
        })
        .unwrap();
    }
}

/// Implementations of private API Core Graphics Services bindings.
/// This is used to change the order of windows.
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
