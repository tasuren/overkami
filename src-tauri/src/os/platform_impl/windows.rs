use anyhow::Context as _;
use tauri::WebviewWindow;
use windows::Win32::{
    Foundation::COLORREF,
    UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos,
        GWL_EXSTYLE, LWA_ALPHA, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_LAYERED,
    },
};

impl super::WindowExt for WebviewWindow {
    fn setup_platform_specific(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn set_opacity(&self, opacity: f64) -> anyhow::Result<()> {
        let hwnd = self.hwnd().unwrap();
        let current = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32;

        if current & WS_EX_LAYERED.0 == 0 {
            let new = current | WS_EX_LAYERED.0;

            unsafe {
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new as _);
            }
        }

        unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), (255. * opacity) as u8, LWA_ALPHA) }
            .context("Failed to set layered window attributes")
    }

    fn set_order_above(&self, relative_to: window_getter::WindowId) -> anyhow::Result<()> {
        let hwnd = self.hwnd().unwrap();

        unsafe {
            SetWindowPos(
                hwnd,
                Some(*relative_to.inner()),
                0,
                0,
                0,
                0,
                SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE,
            )
        }
        .context("Failed to set window position")
    }
}
