use crate::error::{Error, Result};
use std::{
    fmt,
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::Path,
};

use crate::{FromMessagePack, FromYaml};
use serde::Serialize;

/// Trait to load a struct from a file or structured directory.
pub trait LoadYaml: FromYaml {
    /// Reads a file from a given path into new string.
    fn load_string<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<String> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(error!("Error opening file: {:?}", e)),
        };
        let mut s = String::new();
        match f.read_to_string(&mut s) {
            Ok(_) => Ok(s),
            Err(e) => Err(error!("Error reading file to string: {:?}", e)),
        }
    }

    fn load_yml<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<Self> {
        match Self::load_string(path) {
            Ok(source) => Self::from_yml(&source),
            Err(e) => Err(error!("Error loading YAML: {}", e)),
        }
    }
}

/// Trait to load a struct from a file or structured directory.
pub trait LoadMessagePack: FromMessagePack {
    fn load_bytes<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<Vec<u8>> {
        match fs::read(path) {
            Ok(vec) => Ok(vec),
            Err(e) => Err(error!("Error reading file to string: {:?}", e)),
        }
    }
    fn load_mp<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<Self> {
        let bytes = Self::load_bytes(path)?;
        Self::from_mp(&bytes)
    }
}

fn is_yaml<P: AsRef<Path> + fmt::Debug>(path: P) -> bool {
    if path.as_ref().is_dir() {
        return true;
    }
    match path.as_ref().extension() {
        Some(extension) => matches!(extension.to_str(), Some("yml") | Some("yaml")),
        None => false,
    }
}

pub trait Load: LoadMessagePack + LoadYaml {
    fn load<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            return Err(error! {"Path did not exist: {:?}", path});
        }
        if is_yaml(&path) {
            Self::load_yml(path)
        } else {
            Self::load_mp(path)
        }
    }
}

/// Private utility to construct a BufWriter for a file.
fn bufwriter<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<BufWriter<File>> {
    let file = match File::create(path) {
        Ok(f) => f,
        Err(e) => return Err(error!("Failed to create file: {:?}", e)),
    };
    Ok(BufWriter::new(file))
}

/// Trait to save a serializable object to a MessagePack file.
pub trait SaveMessagePack: Serialize {
    fn save_mp<P: AsRef<Path> + fmt::Debug>(&self, path: P) -> Result<()> {
        let buffer = match rmp_serde::to_vec(self) {
            Ok(b) => b,
            Err(e) => return Err(error!("Failed to serialize object: {:?}", e)),
        };
        match bufwriter(path)?.write_all(&buffer) {
            Ok(_) => Ok(()),
            Err(e) => Err(error!("Error writing MessagePack buffer: {:?}", e)),
        }
    }
}

/// Trait to save a serializable object to a YAML file.
pub trait SaveYaml: Serialize {
    fn save_yml<P: AsRef<Path> + fmt::Debug>(&self, path: P) -> Result<()> {
        match serde_yaml::to_writer(bufwriter(path)?, self) {
            Ok(_) => Ok(()),
            Err(e) => Err(error!("Failed to write to file: {:?}", e)),
        }
    }
}

pub trait Save: SaveMessagePack + SaveYaml {
    fn save<P: AsRef<Path> + fmt::Debug>(&self, path: P) -> Result<()> {
        if is_yaml(&path) {
            self.save_yml(path)
        } else {
            self.save_mp(path)
        }
    }
}
