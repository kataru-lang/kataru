use crate::error::{Error, Result};
use serde::de::DeserializeOwned;

/// Trait for parsable expressions.
pub trait FromYaml: DeserializeOwned {
    fn from_yml(text: &str) -> Result<Self> {
        match serde_yaml::from_str(text) {
            Ok(config) => Ok(config),
            Err(e) => Err(error!("Invalid YAML: {}", e)),
        }
    }
}

/// Trait for parsable expressions.
pub trait FromStr<'a> {
    fn from_str(text: &'a str) -> Result<Self>
    where
        Self: std::marker::Sized;
}

/// Trait for extract config/story from MessagePack bytes.
pub trait FromMessagePack: DeserializeOwned {
    fn from_mp(bytes: &[u8]) -> Result<Self> {
        match rmp_serde::from_slice(bytes) {
            Ok(r) => Ok(r),
            Err(e) => Err(error!("{}", e)),
        }
    }
}
