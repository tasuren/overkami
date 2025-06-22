pub mod overlay;
pub mod overlay_host;
pub mod wallpaper_host;

pub use setup::setup_wallpapers;

mod setup {
    use tauri::{async_runtime, Manager};

    use crate::{wallpaper::wallpaper_host::WallpaperHost, ConfigState};

    pub fn setup_wallpapers(app: &tauri::App) {
        let app = app.handle();

        async_runtime::block_on(async move {
            let config = app.state::<ConfigState>();
            let config = config.lock().await;

            let mut hosts = Vec::new();

            for wallpaper in config.wallpapers.iter() {
                let host = WallpaperHost::new(app.clone(), wallpaper.clone()).await;
                hosts.push(host);
            }

            app.manage(hosts);
        });
    }
}
