use crate::{
    error::Error,
    structs::{CharacterData, Config, Params, Passage, Passages},
    traits::{FromYaml, LoadYaml, Merge},
    Map, SetCommand, Value,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt, str::CharIndices};
use std::{iter::Rev, path::Path};

use super::attributes::OptionParams;

lazy_static! {
    static ref SEPARATOR_RE: Regex = Regex::new(r"(\n|\n\r)---").unwrap();
}
pub static GLOBAL: &str = "global";

/// A qualified name is a name in an explicit namespace.
/// If namespace is empty, then this name is global.
pub struct QualifiedName<'a> {
    pub namespace: &'a str,
    pub name: &'a str,
}

impl<'a> QualifiedName<'a> {
    /// Constructs a qualified name while in the context of `namespace`.
    /// This means that if no namespace is specified in `name`, then the qname will have `namespace` as its namespace.
    /// Otherwise it takes the namespace specified in `name`.
    pub fn from(namespace: &'a str, name: &'a str) -> Self {
        let split: Vec<&str> = name.rsplitn(2, ":").collect();
        match split.as_slice() {
            [split_name, explicit_namespace] => Self {
                namespace: explicit_namespace,
                name: split_name,
            },
            _ => Self {
                namespace,
                name: name,
            },
        }
    }

    pub fn resolve(&'a self) -> NamespaceResolver<'a> {
        NamespaceResolver::new(self.namespace)
    }
}
enum ResolverState {
    Start,
    Iter,
    End,
    GlobalOnly,
}
pub struct NamespaceResolver<'a> {
    namespace: &'a str,
    char_indices: Rev<CharIndices<'a>>,
    state: ResolverState,
}
impl<'a> NamespaceResolver<'a> {
    fn new(namespace: &'a str) -> Self {
        Self {
            namespace,
            char_indices: namespace.char_indices().rev(),
            state: if namespace == GLOBAL {
                ResolverState::GlobalOnly
            } else {
                ResolverState::Start
            },
        }
    }
}
impl<'a> Iterator for NamespaceResolver<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            ResolverState::Start => {
                // First `next` should just return the namespace.
                self.state = ResolverState::Iter;
                Some(&self.namespace)
            }
            ResolverState::Iter => {
                // All subsequent `next` calls should return the parent namespaces in order.
                while let Some((i, c)) = self.char_indices.next() {
                    if c == ':' {
                        return Some(&self.namespace[0..i]);
                    }
                }
                self.state = ResolverState::End;
                Some(GLOBAL)
            }
            ResolverState::GlobalOnly => {
                self.state = ResolverState::End;
                Some(GLOBAL)
            }
            ResolverState::End => None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub config: Config,
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
    pub fn attributes(&self) -> &Map<String, Option<OptionParams>> {
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

#[cfg(test)]
mod tests {
    use super::QualifiedName;
    use crate::GLOBAL;

    #[test]
    fn test_namespace_resolution() {
        let namespace = "n1";
        let unqualified_name = "n1:n2:ident";
        let qname = QualifiedName::from(namespace, unqualified_name);
        assert_eq!(qname.namespace, "n1:n2");

        let resolution_order: Vec<&str> = qname.resolve().collect();

        assert_eq!(resolution_order, vec!["n1:n2", "n1", GLOBAL])
    }
}
