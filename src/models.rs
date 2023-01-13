use std::path::Path;
use std::{fs, io};
use std::{
    fs::{read_to_string, write},
    path::PathBuf,
};

use dirs::home_dir;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{error::RuntimeError, parse_internal::from_toml};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UMMConfig {
    pub meta: ConfigMeta,
    pub user: UserConfig,
}

impl UMMConfig {
    pub fn set_ultrakill_path(&mut self, path: PathBuf) -> io::Result<()> {
        self.meta.ultrakill_path = path;
        self.meta.ultrakill_mods = self.meta.ultrakill_path.join("BepInEx").join("UMM Mods");
        self.meta.ultrakill_patterns = self.meta.ultrakill_path.join("Cybergrind").join("Patterns").join("ULTRAMODMANAGER");
        self.save()?;

        Ok(())
    }
}

pub trait Save {
    fn save(&self) -> io::Result<()>;
}

impl Default for UMMConfig {
    /// Panics if the `.ultramodmanager` file is not a directory
    fn default() -> Self {
        let home = home_dir().unwrap();
        let umm_path = home.join(".ultramodmanager");

        if !umm_path.exists() || !umm_path.is_dir() {
            panic!("The ultramodmanager dotfile needs to be a directory.")
        }

        Self {
            meta: ConfigMeta {
                mods_dir: umm_path.join("mods"),
                patterns_dir: umm_path.join("patterns"),
                umm_dir: umm_path,
                ..Default::default()
            },
            user: UserConfig {
                ..Default::default()
            }
        }
    }
}

impl Save for UMMConfig {
    /// Saves the config with any mutable changes you may have implemented
    fn save(&self) -> io::Result<()> {
        write(
            self.meta.umm_dir.join("config.toml"),
            toml::to_string_pretty(self).unwrap(),
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigMeta {
    pub ultrakill_path: PathBuf,
    pub ultrakill_mods: PathBuf,
    pub ultrakill_patterns: PathBuf,
    pub umm_dir: PathBuf,
    pub mods_dir: PathBuf,
    pub patterns_dir: PathBuf,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct UserConfig {
    pub user_token: String,
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
        mod_dir: impl AsRef<Path>,
    ) -> Result<(), RuntimeError> {
        let mod_dir = mod_dir.as_ref();

        if !mod_dir.exists() || !mod_dir.is_dir() {
            return Err(RuntimeError::new(format!(
                "The specified path doesn't exist or is not a directory.",
            )));
        }

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

        let mod_dir_name = format!(
            "{}@{}",
            &parsed_manifest.mod_data.id, &parsed_manifest.mod_data.mod_version
        );
        let dest = config.meta.mods_dir.join(&mod_dir_name);

        if self
            .mods
            .iter()
            .any(|m| {
                m.version == parsed_manifest.mod_data.mod_version
                    && m.id == parsed_manifest.mod_data.id
            })
        {
            return Err(RuntimeError::new(
                "A mod with that id and version already exists locally.",
            ));
        }

        if dest.exists() {
            return Err(RuntimeError::new(
                "A mod with that id and version already exists in the fs.",
            ));
        }

        copy(&mod_dir, &dest).map_err(|e| {
            RuntimeError::new(format!(
                "Unable to copy {} to the ultramodmanager mods directory: {e}",
                &dest.display()
            ))
        })?;

        self.mods.push(ModLockRecord {
            id: parsed_manifest.mod_data.id,
            description: parsed_manifest.mod_data.description,
            name: mod_dir_name,
            version: parsed_manifest.mod_data.mod_version,
            ..Default::default()
        });

        write(
            config.meta.umm_dir.join("ultramodmanager.lock"),
            toml::to_string_pretty(&self).unwrap(),
        )
        .map_err(|_| RuntimeError::new("Failed to write updated lockfile to fs."))?;

        Ok(())
    }

    pub fn install_pattern(
        &mut self,
        config: &UMMConfig,
        version: impl AsRef<str>,
        name: impl AsRef<str>,
        contents: impl AsRef<str>,
    ) -> Result<(), RuntimeError> {
        let base_name = name.as_ref().replace(".cgp", "");
        let base_vers = Version::parse(version.as_ref())
            .map_err(|_| RuntimeError::new("Unable to parse pattern version to semver."))?;
        let orig_name = format!("{base_name}@{base_vers}");
        let mut name = orig_name.clone();
        let mut copy = 0;

        cygrind_utils::validate(&contents)
            .map_err(|e| RuntimeError::new(format!("{name}.cgp validation failed: {e}")))?;

        while self.patterns.iter().map(|p| &p.name).any(|x| x == &name) {
            name = format!("{orig_name}_({copy})");
            copy += 1;
        }

        write(
            config.meta.patterns_dir.join(format!("{}.cgp", &name)),
            contents.as_ref(),
        )
        .map_err(|_| RuntimeError::new("Failed to write pattern file to fs."))?;

        self.patterns.push(PatternLockRecord {
            name,
            version: version.as_ref().into(),
        });

        write(
            config.meta.umm_dir.join("ultramodmanager.lock"),
            toml::to_string_pretty(&self).unwrap(),
        )
        .map_err(|_| RuntimeError::new("Failed to write updated lockfile to fs."))?;

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
    pub version: String,
}

#[cfg(test)]
mod test {
    use crate::manager::init;

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

    #[test]
    fn manifest() {
        let manifest = Manifest::default();
        println!("{}", toml::to_string(&manifest).unwrap())
    }

    #[test]
    fn install_pattern() {
        let (config, mut lock) = init().unwrap();

        lock.install_pattern(&config, "0.1.0", "uwu owo", include_str!("../test-data/test.cgp")).unwrap();
    }

    #[test]
    fn install_mod() {
        let (config, mut lock) = init().unwrap();

        lock.install_mod(&config, "test-data/test-mod").unwrap();
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
            println!(" mkdir: {dest:?}");
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
                        println!("failed: {path:?}");
                    }
                }
            }
        }
    }

    Ok(())
}
