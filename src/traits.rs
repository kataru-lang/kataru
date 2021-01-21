use super::Map;
use crate::ParseError;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

pub trait Mergeable {
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

/// Trait for parsable expressions.
pub trait Parsable<'a> {
    fn parse(text: &'a str) -> Result<Self, ParseError>
    where
        Self: std::marker::Sized;
}

/// Trait for extract config/story from MessagePack bytes.
pub trait Deserializable {
    fn deserialize(bytes: &[u8]) -> Self
    where
        Self: Sized;
}

pub trait Loadable {
    /// Reads a file from a given path into new string.
    fn load_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let mut f = File::open(path)?;
        let mut s = String::new();
        match f.read_to_string(&mut s) {
            Ok(_) => Ok(s),
            Err(e) => Err(e),
        }
    }

    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self>
    where
        Self: Sized;
}
