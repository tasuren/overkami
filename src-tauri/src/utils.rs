use percent_encoding::utf8_percent_encode;

pub use scale_factor::{adjust_position, adjust_size};

/// JavaScriptの`convertFileSrc`APIのRust版。
pub fn convert_file_src(path: impl AsRef<std::path::Path>) -> std::io::Result<String> {
    #[cfg(any(windows, target_os = "android"))]
    let base = "http://asset.localhost/";
    #[cfg(not(any(windows, target_os = "android")))]
    let base = "asset://localhost/";

    let path = dunce::canonicalize(path)?;
    let path = path.to_string_lossy();
    let encoded = utf8_percent_encode(&path, percent_encoding::NON_ALPHANUMERIC);

    Ok(format!("{base}{encoded}"))
}

mod scale_factor {
    use tauri::{LogicalPosition, LogicalSize, WebviewWindow};

    fn scale_factor(window: &WebviewWindow) -> f64 {
        window.scale_factor().expect("Failed to get scale factor")
    }

    pub fn adjust_size(_window: &WebviewWindow, width: f64, height: f64) -> LogicalSize<f64> {
        #[cfg(target_os = "macos")]
        {
            LogicalSize::new(width, height)
        }
        #[cfg(target_os = "windows")]
        {
            tauri::PhysicalSize::new(width, height).to_logical(scale_factor(_window))
        }
    }

    pub fn adjust_position(_window: &WebviewWindow, x: f64, y: f64) -> LogicalPosition<f64> {
        #[cfg(target_os = "macos")]
        {
            LogicalPosition::new(x, y)
        }
        #[cfg(target_os = "windows")]
        {
            tauri::PhysicalPosition::new(x, y).to_logical(scale_factor(_window))
        }
    }
}
