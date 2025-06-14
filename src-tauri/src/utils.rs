use percent_encoding::utf8_percent_encode;

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
