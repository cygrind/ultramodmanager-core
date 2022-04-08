use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

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

    /// YYYY-MM-DD
    pub date: String,

    /// semver
    pub uk_version: String,

    /// semver
    pub mod_version: String,
}

pub type Deserializers = HashMap<&'static str, &'static (dyn Fn(&str) -> Manifest + Send + Sync)>;
pub type Serializers = HashMap<&'static str, &'static (dyn Fn(&Manifest) -> String + Send + Sync)>;

lazy_static! {
    pub static ref DESERIALIZERS: Deserializers = HashMap::from_iter([
        (
            "toml",
            &from_toml as &(dyn Fn(&str) -> Manifest + Send + Sync)
        ),
        (
            "json",
            &from_json as &(dyn Fn(&str) -> Manifest + Send + Sync)
        )
    ]);
    pub static ref SERIALIZERS: Serializers = HashMap::from_iter([
        (
            "toml",
            &to_toml_string as &(dyn Fn(&Manifest) -> String + Send + Sync)
        ),
        (
            "json",
            &to_json5_string as &(dyn Fn(&Manifest) -> String + Send + Sync)
        )
    ]);
}

pub fn from_toml(src: &str) -> Manifest {
    toml::from_str(src).unwrap()
}

pub fn from_json(src: &str) -> Manifest {
    json5::from_str(src).unwrap()
}

pub fn to_toml_string(manifest: &Manifest) -> String {
    toml::to_string(manifest).unwrap()
}

pub fn to_json5_string(manifest: &Manifest) -> String {
    json5::to_string(manifest).unwrap()
}