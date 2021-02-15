use std::{
    fmt,
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};

use crate::error::ParseError;
use serde::Serialize;

/// Trait to load a struct from a file or structured directory.
pub trait Load {
    /// Reads a file from a given path into new string.
    fn load_string<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<String, ParseError> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(perror!("Error opening file: {:?}", e)),
        };
        let mut s = String::new();
        match f.read_to_string(&mut s) {
            Ok(_) => Ok(s),
            Err(e) => return Err(perror!("Error reading file to string: {:?}", e)),
        }
    }

    fn load<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<Self, ParseError>
    where
        Self: Sized;
}

/// Private utility to construct a BufWriter for a file.
fn bufwriter<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<BufWriter<File>, ParseError> {
    let file = match File::create(path) {
        Ok(f) => f,
        Err(e) => return Err(perror!("Failed to create file: {:?}", e)),
    };
    Ok(BufWriter::new(file))
}

/// Trait to save a serializable object to a MessagePack file.
pub trait SaveMessagePack: Serialize {
    fn save_mp<P: AsRef<Path> + fmt::Debug>(&self, path: P) -> Result<(), ParseError> {
        let buffer = match rmp_serde::to_vec(self) {
            Ok(b) => b,
            Err(e) => return Err(perror!("Failed to serialize object: {:?}", e)),
        };
        match bufwriter(path)?.write(&buffer) {
            Ok(_) => Ok(()),
            Err(e) => return Err(perror!("Error writing MessagePack buffer: {:?}", e)),
        }
    }
}

/// Trait to save a serializable object to a YAML file.
pub trait SaveYaml: Serialize {
    fn save_yaml<P: AsRef<Path> + fmt::Debug>(&self, path: P) -> Result<(), ParseError> {
        match serde_yaml::to_writer(bufwriter(path)?, self) {
            Ok(_) => Ok(()),
            Err(e) => return Err(perror!("Failed to write to file: {:?}", e)),
        }
    }
}
