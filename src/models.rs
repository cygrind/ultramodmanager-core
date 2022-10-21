use std::fs;
use std::path::Path;
use std::{
    fs::{read_to_string, write},
    path::PathBuf,
};

#[cfg(unix)]
use std::os::unix;

#[cfg(windows)]
use std::os::windows;

use serde::{Deserialize, Serialize};

use crate::{error::RuntimeError, parse_internal::from_toml};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct UMMConfig {
    pub meta: ConfigMeta,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigMeta {
    pub ultrakill_path: PathBuf,
    pub ultrakill_mods: PathBuf,
    pub ultrakill_patterns: PathBuf,
    pub config_path: PathBuf,
    pub umm_dir: PathBuf,
    pub mods_dir: PathBuf,
    pub patterns_dir: PathBuf,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Manifest {
    #[serde(rename = "mod")]
    pub mod_data: Mod,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub source_url: String,
    pub download_url: String,
    pub checksum: String,

    /// Location of mod icon relative to the mod directory
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
        mod_dir: PathBuf,
    ) -> Result<(), RuntimeError> {
        let manifest_path = mod_dir.join("manifest.toml");

        if !manifest_path.exists() || !manifest_path.is_file() {
            return Err(RuntimeError::new(format!(
                "manifest.toml is not present in mod directory \"{}\".",
                mod_dir.file_name().unwrap().to_string_lossy()
            )));
        }

        let loaded_manifest = read_to_string(manifest_path).map_err(|e| {
            RuntimeError::new(format!("Unable to read ultramodmanager config file: {e}"))
        })?;

        let parsed_manifest = from_toml(&loaded_manifest)
            .map_err(|e| RuntimeError::new(format!("Unable to parse manifest: {e}")))?;

        let mod_dir_name = mod_dir.file_name().unwrap().to_str().unwrap();
        let dest = config.meta.mods_dir.join(mod_dir_name);

        copy(&mod_dir, &dest).map_err(|e| {
            RuntimeError::new(format!(
                "Unable to copy {} to the ultramodmanager mods directory: {e}",
                &dest.display()
            ))
        })?;

        #[cfg(unix)]
        {
            if let Err(e) =
                unix::fs::symlink(&dest, &config.meta.ultrakill_mods.join(&mod_dir_name))
            {
                return Err(RuntimeError::new(format!(
                    "Unable to symlink new mod {} to the ULTRAKILL Mods directory ({:?}): {}",
                    &mod_dir_name, &config.meta.ultrakill_mods, e
                )));
            }
        }

        #[cfg(windows)]
        {
            if let Err(e) =
                windows::fs::symlink_dir(&dest, &config.meta.ultrakill_mods.join(&mod_dir_name))
            {
                return Err(RuntimeError::new(format!(
                    "Unable to symlink new mod {} to the ULTRAKILL Mods directory ({:?}): {}",
                    &mod_dir_name, &config.meta.ultrakill_mods, e
                )));
            }
        }

        self.mods.push(ModLockRecord {
            id: parsed_manifest.mod_data.id,
            description: parsed_manifest.mod_data.description,
            name: parsed_manifest.mod_data.name,
            version: parsed_manifest.mod_data.mod_version,
            ..Default::default()
        });

        Ok(())
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
            .map_err(|e| RuntimeError::new(format!("{name}.cgp validation failed: {}", e)))?;

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
                &config
                    .meta
                    .ultrakill_patterns
                    .join(format!("{}.cgp", &name)),
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
                &config
                    .meta
                    .ultrakill_patterns
                    .join(format!("{}.cgp", &name)),
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
            description: String::new(),
            id: String::new(),
            name: String::new(),
            version: String::new(),
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

    #[test]
    fn umm_config() {
        let config = UMMConfig::default();
        println!("{}", toml::to_string(&config).unwrap())
    }
}

fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}
