use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct UMMConfig {
    pub meta: ConfigMeta,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ConfigMeta {
    #[serde(rename = "ultrakill-path")]
    pub ultrakill_path: String,
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
    pub patterns: Vec<PatternLockRecord>,
    pub mods: Vec<ModLockRecord>,
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
        Self { name: Default::default(), id: Default::default(), description: Default::default(), version: Default::default(), autoload: true }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct PatternLockRecord {
    pub name: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lockfile() {
        let lock = LockFile {
            mods: vec![
                ModLockRecord {
                    ..Default::default()
                };
                2
            ],
            patterns: vec![
                PatternLockRecord {
                    ..Default::default()
                };
                2
            ],
        };

        let st = toml::to_string(&lock).unwrap();
        println!("{st}")
    }
}
