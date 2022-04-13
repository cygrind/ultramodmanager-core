use std::{error::Error, fmt::Display};

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
    pub checksum: String,

    /// RFC 3339
    pub date: String,

    /// semver
    pub uk_version: String,

    /// semver
    pub mod_version: String,
}

pub fn deserialize(format: &str, src: &str) -> Result<Manifest, Box<dyn Error>> {
    match format {
        "toml" => from_toml(src),
        "json" => from_json(src),
        _ => Err(Box::new(ParseError::new("Invalid deserialization type"))),
    }
}

pub fn serialize(format: &str, manifest: Manifest) -> Result<String, Box<dyn Error>> {
    match format {
        "toml" => to_toml_string(&manifest),
        "json" => to_json5_string(&manifest),
        _ => Err(ParseError::new("Invalid deserialization type")),
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn new(message: impl ToString) -> Box<RuntimeError> {
        let message = message.to_string();

        Box::new(RuntimeError { message })
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: impl ToString) -> Box<ParseError> {
        let message = message.to_string();

        Box::new(ParseError { message })
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ParseError {}

pub fn from_toml(src: &str) -> Result<Manifest, Box<dyn Error>> {
    let r = toml::from_str(src)?;
    Ok(r)
}

pub fn from_json(src: &str) -> Result<Manifest, Box<dyn Error>> {
    let r = json5::from_str(src)?;
    Ok(r)
}

pub fn to_toml_string(manifest: &Manifest) -> Result<String, Box<dyn Error>> {
    let r = toml::to_string(manifest)?;
    Ok(r)
}

pub fn to_json5_string(manifest: &Manifest) -> Result<String, Box<dyn Error>> {
    let r = json5::to_string(manifest)?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn errors() {
        let test_toml =
            include_str!("/home/krista/Documents/coding/rust/ultra-mod-manager/test.json");

        let out = from_toml(test_toml);
        match out {
            Ok(manifest) => {
                dbg!(manifest);
            }
            Err(e) => {
                println!("{}", e);
            }
        };
    }
}
