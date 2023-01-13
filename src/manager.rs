use std::{
    fmt::Display,
    fs::{create_dir_all, read_to_string, write},
};

#[cfg(unix)]
use std::os::unix;

#[cfg(windows)]
use std::os::windows;

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
        write!(f, "{self:?}")
    }
}

impl std::error::Error for InitError {}

/// Grabs the .ultramodmanger directory and loads the current config.toml (call this before doing any manager things to grab the config and lock files)
///
/// This creates a new lockfile and config with ALL EMPTY VALUES if they do not already exist
pub fn init() -> Result<(UMMConfig, LockFile), InitError> {
    let home = home_dir().unwrap();
    let umm_dir = home.join(".ultramodmanager");

    if !umm_dir.exists() {
        create_dir_all(&umm_dir)
            .map_err(|_| InitError::new("Unable to create .ultramodmanager directory."))?;
    }

    if !umm_dir.is_dir() {
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

    let patterns_dir = &config.meta.patterns_dir;
    let mods_dir = &config.meta.mods_dir;

    if !patterns_dir.exists() {
        std::fs::create_dir_all(patterns_dir)
            .map_err(|_| InitError::new("Failed to create local patterns directory."))?;
    }

    if !mods_dir.exists() {
        std::fs::create_dir_all(mods_dir)
            .map_err(|_| InitError::new("Failed to create local mods directory."))?;
    }

    if !config.meta.ultrakill_patterns.exists() {
        #[cfg(unix)]
        {
            if config.meta.ultrakill_path.exists() {
                if let Err(e) =
                    unix::fs::symlink(&config.meta.patterns_dir, &config.meta.ultrakill_patterns)
                {
                    return Err(InitError::new(format!(
                            "Unable to symlink ULTRAMODMANAGER patterns directory ({}) to ULTRAKILL Patterns directory ({}): {}",
                            &config.meta.patterns_dir.display(), &config.meta.ultrakill_patterns.display(), e
                        )));
                }
            }
        }

        #[cfg(windows)]
        {
            if config.meta.ultrakill_path.exists() {
                if let Err(e) = windows::fs::symlink_dir(
                    &config.meta.patterns_dir,
                    &config.meta.ultrakill_patterns,
                ) {
                    return Err(InitError::new(format!(
                            "Unable to symlink ULTRAMODMANAGER patterns directory ({}) to ULTRAKILL Patterns directory ({}): {}",
                            &config.meta.patterns_dir.display(), &config.meta.ultrakill_patterns.display(), e
                        )));
                }
            }
        }
    }

    if !config.meta.ultrakill_mods.exists() {
        #[cfg(unix)]
        {
            if config.meta.ultrakill_path.exists() {
                if let Err(e) =
                    unix::fs::symlink(&config.meta.mods_dir, &config.meta.ultrakill_mods)
                {
                    return Err(InitError::new(format!(
                            "Unable to symlink ULTRAMODMANAGER mods directory ({}) to ULTRAKILL Mods directory ({}): {}",
                            &config.meta.mods_dir.display(), &config.meta.ultrakill_mods.display(), e
                        )));
                }
            }
        }

        #[cfg(windows)]
        {
            if config.meta.ultrakill_path.exists() {
                if let Err(e) =
                    windows::fs::symlink_dir(&config.meta.mods_dir, &config.meta.ultrakill_mods)
                {
                    return Err(InitError::new(format!(
                        "Unable to symlink ULTRAMODMANAGER mods directory ({}) to ULTRAKILL Mods directory ({}): {}",
                        &config.meta.mods_dir.display(), &config.meta.ultrakill_mods.display(), e
                    )));
                }
            }
        }
    }

    Ok((config, lock))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init() {
        dbg!(init()).unwrap();
    }
}
