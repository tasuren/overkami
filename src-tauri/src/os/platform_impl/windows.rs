use anyhow::Context as _;
use tauri::WebviewWindow;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

impl crate::os::WebviewWindowPlatformExt for WebviewWindow {
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
        let hwnd_insert_after = unsafe { GetWindow(*relative_to.inner(), GW_HWNDPREV) }
            .context("Failed to get window near target window")?;

        unsafe {
            SetWindowPos(
                hwnd,
                Some(hwnd_insert_after),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            )
        }
        .context("Failed to set window position")
    }
}

impl crate::os::WindowPlatformExt for window_getter::Window {
    fn is_frontmost(&self) -> anyhow::Result<bool> {
        Ok(self.inner().is_foreground())
    }
}

pub fn set_foreground_window(target: &window_getter::Window) -> anyhow::Result<()> {
    let hwnd = target.inner().hwnd();

    unsafe {
        println!("{}", SetForegroundWindow(hwnd).0);
    }

    Ok(())
}

pub mod custom_feature {}
