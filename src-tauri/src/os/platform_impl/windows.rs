use anyhow::Context as _;
use tauri::{LogicalPosition, Manager, WebviewWindow};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

fn get_ex_style(hwnd: HWND) -> anyhow::Result<WINDOW_EX_STYLE> {
    let result = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    anyhow::ensure!(result != 0, "Failed to get current window ex style");
    Ok(WINDOW_EX_STYLE(result as _))
}

fn manage_window_ex_style(
    hwnd: HWND,
    add_mode: bool,
    style: WINDOW_EX_STYLE,
) -> anyhow::Result<()> {
    let current = get_ex_style(hwnd)?;

    let mut result = 1;
    if add_mode && !current.contains(style) {
        result = unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, (current | style).0 as _) }
    }
    if !add_mode && current.contains(style) {
        result = unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, (current & !style).0 as _) };
    }

    anyhow::ensure!(result != 0, "Failed to set window ex style");

    Ok(())
}

struct DiffPos {
    x: f64,
    y: f64,
}

impl crate::os::WebviewWindowPlatformExt for WebviewWindow {
    fn setup_platform_specific(&self) -> anyhow::Result<()> {
        let hwnd = self.hwnd().unwrap();

        manage_window_ex_style(hwnd, true, WS_EX_LAYERED)?;

        // Get window invisible border for resizing.
        // We're going to use this as offset when window moving.
        let scale_factor = self.scale_factor().context("Failed to get scale factor")?;
        let inner_pos = self
            .inner_position()
            .context("Failed to get inner position")?
            .to_logical::<f64>(scale_factor);
        let outer_pos = self
            .outer_position()
            .context("Failed to get outer position")?
            .to_logical::<f64>(scale_factor);

        self.manage(DiffPos {
            x: outer_pos.x - inner_pos.x,
            y: outer_pos.y - inner_pos.y,
        });

        Ok(())
    }

    fn set_opacity(&self, opacity: f64) -> anyhow::Result<()> {
        let hwnd = self.hwnd().unwrap();

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

    fn merge_always_on_top(&self, top: bool) -> anyhow::Result<()> {
        let hwnd = self.hwnd().unwrap();

        manage_window_ex_style(hwnd, top, WS_EX_TOPMOST)?;

        unsafe {
            SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            )
        }
        .context("Failed to set window topmost")
    }

    fn merge_ignore_cursor_events(&self, ignore: bool) -> anyhow::Result<()> {
        let hwnd = self.hwnd().unwrap();

        manage_window_ex_style(hwnd, ignore, WS_EX_TRANSPARENT)
    }

    fn set_position_with_adjustment(&self, x: f64, y: f64) -> anyhow::Result<()> {
        let diff = self.state::<DiffPos>();

        let pos = LogicalPosition::new(x + diff.x, y + diff.y);
        self.set_position(pos)?;

        Ok(())
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
        let _ = SetForegroundWindow(hwnd);
    }

    Ok(())
}

pub mod custom_feature {}
