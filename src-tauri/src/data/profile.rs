use std::{
    fs::read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::error::FileError;
use crate::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct ProfileData {
    
}

pub struct Profile {
    pub path: PathBuf,
    data: ProfileData,
}

impl AsRef<ProfileData> for Profile {
    fn as_ref(&self) -> &ProfileData {
        &self.data
    }
}

impl Profile {
    pub fn new(app_profile_dir: impl AsRef<Path>) -> Result<Self, FileError> {
        let path = app_profile_dir.as_ref().to_path_buf();

        Ok(Self {
            data: serde_json::from_slice(&read(&path).map_err(|e| FileError::IO {
                source: e,
                path: path.clone(),
            })?)
            .context("Failed to parse the Profileuration file.")?,
            path,
        })
    }
}
