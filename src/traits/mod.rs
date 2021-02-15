use super::Map;
use crate::ParseError;

mod file;
mod text;

pub use file::{Load, SaveMessagePack, SaveYaml};
pub use text::{FromMessagePack, FromStr, FromYaml};

/// Trait to merge two objects together. Used for maps.
pub trait Merge {
    fn copy_keys<V>(map: &Map<String, V>) -> Vec<String> {
        let mut keys: Vec<String> = Vec::with_capacity(map.len());
        for key in map.keys() {
            keys.push(key.to_string());
        }
        keys
    }
    fn merge(&mut self, other: &mut Self) -> Result<(), ParseError>
    where
        Self: std::marker::Sized;
}
