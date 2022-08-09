use std::{fs::write, os::unix, path::PathBuf};

#[cfg(windows)]
use std::os::windows;

use serde::{Deserialize, Serialize};

use crate::error::RuntimeError;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct UMMConfig {
    pub meta: ConfigMeta,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ConfigMeta {
    #[serde(rename = "ultrakill-path")]
    pub ultrakill_path: PathBuf,

    #[serde(rename = "ultrakill-mods")]
    pub ultrakill_mods: PathBuf,

    #[serde(skip)]
    pub(crate) ultrakill_patterns: PathBuf,

    #[serde(skip)]
    pub(crate) config_path: PathBuf,

    #[serde(skip)]
    pub(crate) umm_dir: PathBuf,

    #[serde(skip)]
    pub(crate) mods_dir: PathBuf,

    #[serde(skip)]
    pub(crate) patterns_dir: PathBuf,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Manifest {
    #[serde(rename = "mod")]
    pub mod_data: Mod,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub source_url: String,
    pub download_url: String,
    pub checksum: String,

    /// Location of mod icon relative to the .ultramodmanager directory
    pub icon_path: String,

    /// RFC 3339
    pub date: String,

    /// semver
    pub uk_version: String,

    /// semver
    pub mod_version: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct LockFile {
    pub mods: Vec<ModLockRecord>,
    pub patterns: Vec<PatternLockRecord>,
}

impl LockFile {
    pub fn install_mod(
        &mut self,
        config: &UMMConfig,
        mod_path: PathBuf,
    ) -> Result<(), RuntimeError> {
        todo!()
    }

    pub fn install_pattern<S: AsRef<str>>(
        &mut self,
        config: &UMMConfig,
        name: S,
        contents: S,
    ) -> Result<(), RuntimeError> {
        let orig_name = name.as_ref().replace(".cgp", "");
        let mut name = orig_name.clone();
        let mut copy = 0;

        cygrind_utils::validate(&contents)
            .map_err(|e| RuntimeError::new(format!("{name}.cgp validation failed: {}", e.0)))?;

        while self.patterns.iter().map(|p| &p.name).any(|x| x == &name) {
            name = format!("{orig_name}_({copy})");
            copy += 1;
        }

        let _ = write(
            config.meta.patterns_dir.join(format!("{}.cgp", &name)),
            contents.as_ref(),
        );

        #[cfg(unix)]
        {
            if let Err(e) = unix::fs::symlink(
                &config.meta.patterns_dir.join(format!("{}.cgp", &name)),
                &config.meta.ultrakill_patterns.join(format!("{}.cgp", &name)),
            ) {
                return Err(RuntimeError::new(format!(
                    "Unable to symlink {}.cgp to the ULTRAKILL Patterns directory ({:?}): {}",
                    &name, &config.meta.ultrakill_patterns, e
                )));
            }
        }

        #[cfg(windows)]
        {
            if let Err(e) = windows::fs::symlink_file(
                &config.meta.patterns_dir.join(format!("{}.cgp", &name)),
                &config.meta.ultrakill_patterns.join(format!("{}.cgp", &name)),
            ) {
                return Err(RuntimeError::new(format!(
                    "Unable to symlink {}.cgp to the ULTRAKILL Patterns directory ({:?}): {}",
                    &name, &config.meta.ultrakill_patterns, e
                )));
            }
        }

        self.patterns.push(PatternLockRecord { name });

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModLockRecord {
    pub name: String,
    pub id: String,
    pub description: String,
    pub version: String,
    pub autoload: bool,
}

impl Default for ModLockRecord {
    fn default() -> Self {
        Self {
            name: Default::default(),
            id: Default::default(),
            description: Default::default(),
            version: Default::default(),
            autoload: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct PatternLockRecord {
    pub name: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lockfile() {
        let lock = LockFile::default();

        let st = toml::to_string(&lock).unwrap();
        println!("{st}")
    }
}
