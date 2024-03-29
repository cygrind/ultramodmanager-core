use crate::{error::ParseError, models::Manifest};

pub fn from_toml(src: &str) -> Result<Manifest, ParseError> {
    let r = toml::from_str(src).map_err(|e| ParseError::new(e.to_string()))?;
    Ok(r)
}

pub fn to_toml_string(manifest: &Manifest) -> Result<String, ParseError> {
    let r = toml::to_string(manifest).map_err(|e| ParseError::new(e.to_string()))?;
    Ok(r)
}
