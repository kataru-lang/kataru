use crate::{
    error::Error,
    structs::{CharacterData, Config, Params, Passage, Passages},
    traits::{FromYaml, LoadYaml, Merge},
    Map, SetCommand, Value,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

lazy_static! {
    static ref SEPARATOR_RE: Regex = Regex::new(r"(\n|\n\r)---").unwrap();
}
pub static GLOBAL: &str = "global";

/// A qualified name is a name in an explicit namespace.
/// If namespace is empty, then this name is global.
pub struct QualifiedName {
    pub namespace: String,
    pub name: String,
}

impl QualifiedName {
    /// Constructs a qualified name while in the context of `namespace`.
    /// This means that if no namespace is specified in `name`, then the qname will have `namespace` as its namespace.
    /// Otherwise it takes the namespace specified in `name`.
    pub fn from(namespace: &str, name: &str) -> Self {
        let split: Vec<&str> = name.rsplitn(2, ":").collect();
        match split.as_slice() {
            [split_name, explicit_namespace] => Self {
                namespace: explicit_namespace.to_string(),
                name: split_name.to_string(),
            },
            _ => Self {
                namespace: namespace.to_string(),
                name: name.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Section {
    config: Config,
    pub passages: Passages,
}

impl<'a> Section {
    #[cfg(test)]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            passages: Passages::new(),
        }
    }

    #[inline]
    pub fn has_character(&self, character: &str) -> bool {
        self.config.characters.contains_key(character)
    }

    #[inline]
    pub fn state(&self) -> &Map<String, Value> {
        &self.config.state
    }

    #[inline]
    pub fn attributes(&self) -> &Map<String, Option<String>> {
        &self.config.attributes
    }

    #[inline]
    pub fn on_exit(&self) -> &Option<SetCommand> {
        &self.config.on_exit
    }

    #[inline]
    pub fn on_enter(&self) -> &Option<SetCommand> {
        &self.config.on_enter
    }

    #[inline]
    pub fn passage(&'a self, name: &str) -> Option<&'a Passage> {
        self.passages.get(name)
    }

    #[inline]
    pub fn namespace(&'a self) -> &str {
        &self.config.namespace
    }

    #[inline]
    pub fn params(&'a self, name: &str) -> Option<&'a Option<Params>> {
        self.config.commands.get(name)
    }

    #[inline]
    pub fn character(&'a self, name: &str) -> Option<&'a Option<CharacterData>> {
        self.config.characters.get(name)
    }

    #[inline]
    pub fn value(&'a self, name: &str) -> Option<&'a Value> {
        self.config.state.get(name)
    }
}

impl Merge for Section {
    fn merge(&mut self, other: &mut Self) -> Result<(), Error> {
        self.config.merge(&mut other.config)?;
        self.passages.merge(&mut other.passages)?;
        Ok(())
    }
}

impl FromYaml for Section {}

impl LoadYaml for Section {
    fn load_yml<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<Self, Error> {
        let source = Self::load_string(path)?;
        let split: Vec<&str> = SEPARATOR_RE.split(&source).collect();
        match &split[..] {
            [config_str, passages_str] => Ok(Self {
                config: Config::from_yml(config_str)?,
                passages: Passages::from_yml(passages_str)?,
            }),
            [config_str] => Ok(Self {
                config: Config::from_yml(config_str)?,
                passages: Passages::new(),
            }),
            _ => Err(error!("Unable to parse file.")),
        }
    }
}
