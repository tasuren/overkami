pub use application::*;
pub use wallpaper::*;

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

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum WallpaperSource {
        RemoteWebPage { url: String },
        LocalWebPage { path: String },
        Picture { src: String },
        Video { src: String },
    }

    #[derive(PartialEq, Eq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum StringFilterStrategy {
        Prefix,
        Suffix,
        Infix,
        Whole,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum Filter {
        WindowName {
            name: String,
            strategy: StringFilterStrategy,
        },
    }

    #[derive(Serialize, Deserialize)]
    pub struct Wallpaper {
        pub name: String,
        pub application: super::Application,
        pub filters: Vec<Filter>,
        pub source: WallpaperSource,
    }
}
