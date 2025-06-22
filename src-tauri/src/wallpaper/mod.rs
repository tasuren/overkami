pub mod overlay;
pub mod overlay_host;
pub mod wallpaper_host;

pub use setup::setup_wallpapers;

mod setup {
    use std::collections::HashSet;

    use tauri::{async_runtime, Manager};

    use crate::{
        os::{
            application_monitor::{get_application_processes, ApplicationProcess},
            windows::get_windows,
        },
        wallpaper::wallpaper_host::WallpaperHost,
        ConfigState,
    };

    pub fn setup_wallpapers(app: &tauri::App) {
        let app = app.handle();

        async_runtime::block_on(async move {
            let config = app.state::<ConfigState>();
            let config = config.lock().await;

            let mut hosts = Vec::new();
            let processes: Vec<ApplicationProcess> = get_application_processes(|_| true);
            let windows = get_windows().await;

            for wallpaper in config.wallpapers.iter() {
                let host = WallpaperHost::new(app.clone(), wallpaper.clone()).await;

                let mut pids = HashSet::new();
                for process in processes.iter().rev() {
                    if wallpaper.application.path == process.path {
                        pids.insert(process.pid);
                    }
                }

                if pids.is_empty() {
                    continue;
                } else {
                    for pid in pids {
                        host.setup_initial_windows(pid, &windows).await;
                    }
                }

                hosts.push(host);
            }

            app.manage(hosts);
        });
    }
}
