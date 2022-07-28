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

/// Grabs the .ultramodmanger directory and loads the current config.toml
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

    if !config_path.exists() || !config_path.is_file() {
        return Err(InitError::new("Ultramodmanger config file missing."));
    }

    let config = read_to_string(config_path)
        .map_err(|_| InitError::new("Unable to read ultramodmanager config file."))?;

    let loaded_config = toml::from_str::<UMMConfig>(&*config)
        .map_err(|_| InitError::new("Unable to parse ultramodmanager config file."))?;

    let loaded_lockfile = if !lockfile_path.exists() || !lockfile_path.is_file() {
        let lockfile = LockFile::default();
        let lock_string = toml::to_string_pretty(&lockfile).unwrap();
        write(&lockfile_path, lock_string).unwrap();

        lockfile
    } else {
        let lockfile = read_to_string(lockfile_path)
            .map_err(|_| InitError::new("Unable to read ultramodmanager lockfile."))?;

        toml::from_str::<LockFile>(&*lockfile)
            .map_err(|_| InitError::new("Unable to parse ultramodmanager config file."))?
    };

    Ok((loaded_config, loaded_lockfile))
}
