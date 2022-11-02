use std::{
    fmt::Display,
    fs::{read_to_string, write},
};

use dirs::home_dir;

use crate::models::{LockFile, UMMConfig};

#[derive(Debug)]
pub struct InitError {
    pub message: String,
}

impl InitError {
    pub fn new<S: ToString>(s: S) -> Self {
        InitError {
            message: s.to_string(),
        }
    }
}

impl Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for InitError {}

/// Grabs the .ultramodmanger directory and loads the current config.toml (call this before doing any manager things to grab the config and lock files)
///
/// This creates a new lockfile and config with ALL EMPTY VALUES if they do not already exist
pub fn init() -> Result<(UMMConfig, LockFile), InitError> {
    let home = home_dir().unwrap();
    let umm_dir = home.join(".ultramodmanager");

    if !umm_dir.exists() || !umm_dir.is_dir() {
        return Err(InitError::new(
            "The ultramodmanager dotfile needs to be a directory.",
        ));
    }

    let config_path = umm_dir.join("config.toml");
    let lockfile_path = umm_dir.join("ultramodmanager.lock");

    let config = if config_path.exists() && config_path.is_file() {
        let config = read_to_string(&config_path)
            .map_err(|_| InitError::new("Unable to read ultramodmanager config file."))?;

        toml::from_str(&config)
            .map_err(|_| InitError::new("Unable to parse ultramodmanager config file."))?
    } else {
        let out = UMMConfig::default();
        let config = toml::to_string_pretty(&out).unwrap();

        write(config_path, config)
            .map_err(|_| InitError::new("Failed to write default config file to fs."))?;

        out
    };

    let lock = if lockfile_path.exists() && lockfile_path.is_file() {
        let lockfile = read_to_string(lockfile_path)
            .map_err(|_| InitError::new("Unable to read ultramodmanager lockfile."))?;

        toml::from_str(&lockfile)
            .map_err(|_| InitError::new("Unable to parse ultramodmanager config file."))?
    } else {
        let out = LockFile::default();
        let lockfile = toml::to_string_pretty(&out).unwrap();

        write(lockfile_path, lockfile)
            .map_err(|_| InitError::new("Failed to write default lockfile to fs."))?;

        out
    };

    Ok((config, lock))
}
