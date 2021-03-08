use super::{Map, QualifiedName, State, Story, Value};
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
    #[serde(default)]
    pub snapshots: Map<String, Vec<Position>>,
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

    // Updates `state[var] = val` iff `var` not already in `state`.
    fn default_val(state: &mut State, var: &str, val: &Value) {
        if state.get(var).is_none() {
            state.insert(var.to_string(), val.clone());
        }
    }

    pub fn init_state(&mut self, story: &Story) {
        for (namespace, section) in story {
            if self.state.get(namespace).is_none() {
                self.state.insert(namespace.to_string(), State::default());
            }

            let namespace_state = self.state.get_mut(namespace).unwrap();
            for (var, val) in &section.config.state {
                if var.contains("$passage") {
                    for passage in section.passages.keys() {
                        let replaced = format!("{}{}", passage, &var["$passage".len()..]);
                        Self::default_val(namespace_state, &replaced, &val);
                    }
                } else {
                    Self::default_val(namespace_state, &var, &val);
                }
            }
        }
    }

    pub fn save_snapshot(&mut self, name: &str) {
        self.stack.push(self.position.clone());
        self.snapshots.insert(name.to_string(), self.stack.clone());
    }

    pub fn load_snapshot(&mut self, name: &str) -> Result<()> {
        if let Some(stack) = self.snapshots.remove(name) {
            self.stack = stack;
            if let Some(position) = self.stack.pop() {
                self.position = position;
                Ok(())
            } else {
                Err(error!("Snapshot named '{}' was empty", name))
            }
        } else {
            Err(error!("No snapshot named '{}'", name))
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
