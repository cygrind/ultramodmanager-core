pub mod models;
pub mod error;
mod parse_internal;

use models::Manifest;
use error::ParseError;
use parse_internal::*;

pub fn deserialize(format: &str, src: &str) -> Result<Manifest, ParseError> {
    match format {
        "toml" => from_toml(src),
        "json" => from_json(src),
        _ => Err(ParseError::new("Invalid deserialization type")),
    }
}

pub fn serialize(format: &str, manifest: Manifest) -> Result<String, ParseError> {
    match format {
        "toml" => to_toml_string(&manifest),
        "json" => to_json5_string(&manifest),
        _ => Err(ParseError::new("Invalid deserialization type")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
