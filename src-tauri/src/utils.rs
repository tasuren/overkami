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

/// Adjusts the size and position of a window based on the scale factor of the current platform.
/// Windows will return physical window rect so we need to convert it to logical version on windows.
mod scale_factor {
    use tauri::{LogicalPosition, LogicalSize};

    pub fn adjust_size(_scale_factor: f64, width: f64, height: f64) -> LogicalSize<f64> {
        #[cfg(target_os = "macos")]
        {
            LogicalSize::new(width, height)
        }
        #[cfg(target_os = "windows")]
        {
            tauri::PhysicalSize::new(width, height).to_logical(_scale_factor)
        }
    }

    pub fn adjust_position(_scale_factor: f64, x: f64, y: f64) -> LogicalPosition<f64> {
        #[cfg(target_os = "macos")]
        {
            LogicalPosition::new(x, y)
        }
        #[cfg(target_os = "windows")]
        {
            tauri::PhysicalPosition::new(x, y).to_logical(_scale_factor)
        }
    }
}
