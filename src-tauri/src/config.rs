pub use model::*;
pub use setup::setup_config;
pub use state::{ConfigPathState, ConfigState};

mod model {
    pub use application::*;
    use serde::{Deserialize, Serialize};
    pub use wallpaper::*;

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Config {
        pub version: String,
        pub wallpapers: Vec<Wallpaper>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                version: VERSION.to_owned(),
                wallpapers: Vec::new(),
            }
        }
    }

    mod application {
        use std::path::PathBuf;

        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Application {
            pub name: Option<String>,
            pub path: PathBuf,
        }

        impl PartialEq for Application {
            fn eq(&self, other: &Self) -> bool {
                self.path == other.path
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
        pub struct Wallpaper {
            pub name: String,
            pub application: super::Application,
            pub filters: Vec<Filter>,
            pub source: WallpaperSource,
            pub opacity: f64,
        }
    }
}

mod state {
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

    pub fn set_config_path_state(app: &tauri::AppHandle, path: std::path::PathBuf) {
        app.manage(ConfigPathState(path));
    }
}

mod setup {
    use std::path::PathBuf;

    use tauri::{App, Manager};
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

    use super::Config;

    pub fn setup_config(app: &App) {
        let path = app.path();

        let config_path = if cfg!(debug_assertions) {
            PathBuf::from("../mock-env/config")
        } else {
            path.app_config_dir()
                .expect("Failed to get app config directory")
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

    fn failed_to_prepare_config_dir(app: &App, error: String) -> ! {
        app.dialog()
            .message(format!(
                "設定ファイルを配置するフォルダの作成に失敗しました。\n詳細: {error}"
            ))
            .buttons(MessageDialogButtons::Ok)
            .blocking_show();

        std::process::exit(1);
    }

    fn failed_to_read_config_file(app: &App, error: String) -> ! {
        app.dialog()
            .message(format!(
                "設定ファイルの読み込みに失敗しました。\n詳細: {error}"
            ))
            .buttons(MessageDialogButtons::Ok)
            .blocking_show();

        std::process::exit(1);
    }

    fn failed_to_parse_config_file(app: &App, error: String) -> ! {
        app.dialog()
            .message(format!(
                "設定ファイルのデータ構造が不適切です。\n詳細: {error}"
            ))
            .buttons(MessageDialogButtons::Ok)
            .blocking_show();

        std::process::exit(1);
    }
}
