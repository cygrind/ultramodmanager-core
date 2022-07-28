use std::{fmt::Display, fs::read_to_string};

use dirs::home_dir;

use crate::models::UMMConfig;

#[derive(Debug)]
pub struct InitError {
    message: String,
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
pub fn init() -> Result<UMMConfig, InitError> {
    let home = home_dir().unwrap();
    let umm_dir = home.join(".ultramodmanager");

    if !umm_dir.exists() || !umm_dir.is_dir() {
        return Err(InitError::new(
            "The ultramodmanager dotfile needs to be a directory.",
        ));
    }

    let config_path = umm_dir.join("config.toml");

    if !config_path.exists() || !config_path.is_file() {
        return Err(InitError::new("Ultramodmanger config file missing."));
    }

    let config = read_to_string(config_path)
        .map_err(|_| InitError::new("Unable to read ultramodmanager config file."))?;

    toml::from_str::<UMMConfig>(&*config)
        .map_err(|_| InitError::new("Unable to parse ultramodmanager config file."))
}
