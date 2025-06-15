use objc2::rc::Retained;
use objc2_app_kit::NSWindow;
use tauri::WebviewWindow;

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
}
