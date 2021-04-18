use crate::error::{Error, Result};
use serde::de::DeserializeOwned;

use serde_yaml;
/// Trait for parsable expressions.
pub trait FromYaml: DeserializeOwned + Default {
    fn from_yml(text: &str) -> Result<Self> {
        if text.trim_start().is_empty() {
            return Ok(Self::default());
        }

        match serde_yaml::from_str(text) {
            Ok(config) => Ok(config),
            Err(e) => {
                if format!("{:?}", e) == "EndOfStream" {
                    Ok(Self::default())
                } else {
                    Err(error!("Invalid YAML: {}", e))
                }
            }
        }
    }
}

/// Trait for parsable expressions.
pub trait FromStr<'a> {
    fn from_str(text: &'a str) -> Result<Self>
    where
        Self: std::marker::Sized;
}

/// Trait for parsable expressions.
pub trait IntoStr {
    fn into_str(&self) -> &str;
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
