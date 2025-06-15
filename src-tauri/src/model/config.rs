pub use application::*;
use serde::{Deserialize, Serialize};
pub use wallpaper::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
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

    #[derive(Debug, Serialize, Deserialize)]
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
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum WallpaperSource {
        RemoteWebPage { url: String },
        LocalWebPage { path: String },
        Picture { src: String },
        Video { src: String },
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum StringFilterStrategy {
        Prefix,
        Suffix,
        Contains,
        Exact,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum Filter {
        WindowName {
            name: String,
            strategy: StringFilterStrategy,
        },
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Wallpaper {
        pub name: String,
        pub application: super::Application,
        pub filters: Vec<Filter>,
        pub source: WallpaperSource,
    }
}
