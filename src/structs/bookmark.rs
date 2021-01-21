use super::story::{get_from, resolve_namespace};
use super::{Entry, Map, State, Story, Value};
use crate::error::ParseError;
use crate::traits::{Deserializable, Loadable, Parsable};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

/// All data necessary to find your place in the story.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct Bookmark {
    #[serde(default)]
    pub state: Map<String, State>,
    #[serde(default)]
    pub passage: String,
    #[serde(default)]
    pub line: usize,
    #[serde(default)]
    pub namespace: String,
}

impl<'a> Bookmark {
    pub fn val(&'a self, var: &str) -> Option<&'a Value> {
        let (full_namespace, base_name) = resolve_namespace(&self.namespace, var);
        get_from(
            self.state.get(full_namespace)?,
            self.state.get("")?,
            base_name,
        )
        .0
        // self.state.get(&self.namespace).unwrap().get(var).unwrap()
    }

    pub fn state(&'a mut self) -> &'a mut State {
        self.state.get_mut(&self.namespace).unwrap()
    }

    pub fn root_state(&'a mut self) -> &'a mut State {
        self.state.get_mut("").unwrap()
    }

    pub fn init_state(&mut self, story: &Story) {
        for (namespace, section) in story {
            match self.state.entry(namespace.clone()) {
                Entry::Occupied(o) => {
                    let state = o.into_mut();
                    for (var, val) in &section.config.state {
                        state.entry(var.clone()).or_insert(val.clone());
                    }
                }
                Entry::Vacant(v) => {
                    v.insert(section.config.state.clone());
                }
            }
        }
    }
}

impl Deserializable for Bookmark {
    fn deserialize(bytes: &[u8]) -> Self {
        rmp_serde::from_slice(bytes).unwrap()
    }
}

impl Loadable for Bookmark {
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        match Self::load_string(path) {
            Ok(source) => Ok(Self::parse(&source).unwrap()),
            _ => Ok(Self::default()),
        }
    }
}

impl<'a> Parsable<'a> for Bookmark {
    fn parse(text: &'a str) -> Result<Self, ParseError> {
        match serde_yaml::from_str(text) {
            Ok(config) => Ok(config),
            Err(e) => Err(perror!("{}", e)),
        }
    }
}
