use super::{Entry, Map, QualifiedName, State, Story, Value};
use crate::error::{Error, Result};
use crate::{
    traits::{FromMessagePack, FromYaml, LoadYaml, SaveMessagePack},
    Load, LoadMessagePack, Save, SaveYaml, GLOBAL,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct Position {
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    pub passage: String,
    #[serde(default)]
    pub line: usize,
}

/// All data necessary to find your place in the story.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct Bookmark {
    #[serde(default)]
    pub state: Map<String, State>,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub stack: Vec<Position>,
}

impl<'a> Bookmark {
    pub fn value(&'a self, var: &str) -> Result<&'a Value> {
        let qname = QualifiedName::from(&self.position.namespace, var);
        if let Some(section) = self.state.get(&qname.namespace) {
            if let Some(val) = section.get(&qname.name) {
                return Ok(val);
            }
        } else {
            return Err(error!("No state for namespace '{}'", &qname.namespace));
        }

        if let Some(section) = self.state.get(GLOBAL) {
            if let Some(val) = section.get(&qname.name) {
                return Ok(val);
            }
        } else {
            return Err(error!("No state for root namespace"));
        }

        // Return error if there is no passage name in either namespace.
        Err(error!(
            "Variable '{}' could not be found in '{}' nor root namespace state",
            qname.name, qname.namespace
        ))
    }

    pub fn state(&'a mut self) -> Result<&'a mut State> {
        match self.state.get_mut(&self.position.namespace) {
            Some(state) => Ok(state),
            None => Err(error!("Invalid namespace {}", &self.position.namespace)),
        }
    }

    pub fn root_state(&'a mut self) -> Result<&'a mut State> {
        match self.state.get_mut(GLOBAL) {
            Some(state) => Ok(state),
            None => Err(error!("No root namesapce")),
        }
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

impl FromYaml for Bookmark {}
impl FromMessagePack for Bookmark {}
impl SaveYaml for Bookmark {}
impl SaveMessagePack for Bookmark {}
impl Save for Bookmark {}
impl LoadYaml for Bookmark {}
impl LoadMessagePack for Bookmark {}
impl Load for Bookmark {}
