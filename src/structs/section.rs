use crate::error::ParseError;
use crate::structs::{Config, Passages};
use crate::traits::{Loadable, Mergeable, Parsable};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub config: Config,
    pub passages: Passages,
}
impl Mergeable for Section {
    fn merge(&mut self, other: &mut Self) -> Result<(), ParseError> {
        self.config.merge(&mut other.config)?;
        self.passages.merge(&mut other.passages)?;
        Ok(())
    }
}

impl Loadable for Section {
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let source = Self::load_string(path)?;
        let split: Vec<&str> = source.split("---").collect();
        if let [config_str, passages_str] = &split[1..] {
            Ok(Self {
                config: Config::parse(config_str).unwrap(),
                passages: Passages::parse(passages_str).unwrap(),
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to parse file.",
            ))
        }
    }
}
