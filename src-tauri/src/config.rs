pub use model::*;
pub use setup::setup_config;

mod model {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    pub use wallpaper::*;

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Config {
        pub version: String,
        pub open_window_on_startup: bool,
        pub wallpapers: HashMap<Uuid, Wallpaper>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                version: VERSION.to_owned(),
                open_window_on_startup: true,
                wallpapers: HashMap::new(),
            }
        }
    }

    mod wallpaper {
        use std::path::PathBuf;

        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type")]
        pub enum WallpaperSource {
            RemoteWebPage { location: String },
            LocalWebPage { location: PathBuf },
            YouTube { location: String },
            Picture { location: PathBuf },
            Video { location: PathBuf },
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum StringFilterStrategy {
            Prefix,
            Suffix,
            Contains,
            Exact,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type")]
        pub enum Filter {
            WindowName {
                name: String,
                strategy: StringFilterStrategy,
            },
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Wallpaper {
            pub name: String,
            pub application_name: String,
            pub filters: Vec<Filter>,
            pub source: WallpaperSource,
            pub opacity: f64,
        }
    }
}

pub mod state {
    use std::path::PathBuf;

    use tauri::{async_runtime::Mutex, Manager};

    use super::Config;

    pub type ConfigState = Mutex<Config>;

    pub fn set_config_state(app: &tauri::AppHandle, config: Config) {
        app.manage(Mutex::new(config));
    }

    pub struct ConfigPathState(PathBuf);

    impl std::ops::Deref for ConfigPathState {
        type Target = PathBuf;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    pub(super) fn set_config_path_state(app: &tauri::AppHandle, path: std::path::PathBuf) {
        app.manage(ConfigPathState(path));
    }
}

mod setup {
    use std::path::PathBuf;

    use tauri::{App, Manager};
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

    use super::Config;

    pub fn setup_config(app: &App) {
        let path = app.path();

        let config_path = if cfg!(debug_assertions) {
            PathBuf::from("../mock-env/config")
        } else {
            path.app_config_dir()
                .unwrap_or_else(|e| failed_to_get_app_config_directory(app, e.to_string()))
        };

        if !config_path.exists() {
            if let Err(error) = std::fs::create_dir_all(&config_path) {
                failed_to_prepare_config_dir(app, error.to_string());
            };
        }

        let config_path = config_path.join("config.json");

        let config = if config_path.exists() {
            let raw = match std::fs::read(&config_path) {
                Ok(raw) => raw,
                Err(error) => failed_to_read_config_file(app, error.to_string()),
            };

            match serde_json::from_slice(&raw) {
                Ok(config) => config,
                Err(error) => failed_to_parse_config_file(app, error.to_string()),
            }
        } else {
            Config::default()
        };

        let handle = app.handle();
        super::state::set_config_path_state(handle, config_path);
        super::state::set_config_state(handle, config);
    }

    fn error_message(app: &App, message: &str) -> ! {
        app.dialog()
            .message(message)
            .kind(MessageDialogKind::Error)
            .buttons(MessageDialogButtons::Ok)
            .blocking_show();

        std::process::exit(1);
    }

    fn failed_to_get_app_config_directory(app: &App, error: String) -> ! {
        error_message(
            app,
            &format!("設定ファイルを配置するフォルダの取得に失敗しました。\n詳細: {error}"),
        )
    }

    fn failed_to_prepare_config_dir(app: &App, error: String) -> ! {
        error_message(
            app,
            &format!("設定ファイルを配置するフォルダの作成に失敗しました。\n詳細: {error}"),
        )
    }

    fn failed_to_read_config_file(app: &App, error: String) -> ! {
        error_message(
            app,
            &format!("設定ファイルの読み込みに失敗しました。\n詳細: {error}"),
        )
    }

    fn failed_to_parse_config_file(app: &App, error: String) -> ! {
        error_message(
            app,
            &format!("設定ファイルのデータ構造が不適切です。\n詳細: {error}"),
        )
    }
}
