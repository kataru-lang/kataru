use super::{Entry, Map, QualifiedName, State, Story, Value};
use crate::{
    traits::{FromMessagePack, FromYaml, LoadYaml, SaveMessagePack},
    LoadMessagePack, SaveYaml,
};
use serde::{Deserialize, Serialize};

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
    pub fn value(&'a self, var: &str) -> Option<&'a Value> {
        let qname = QualifiedName::from(&self.namespace, var);
        match self.state.get(&qname.namespace)?.get(&qname.name) {
            Some(data) => Some(data),
            None => self.state.get("")?.get(&qname.name),
        }
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

impl FromYaml for Bookmark {}
impl FromMessagePack for Bookmark {}
impl SaveYaml for Bookmark {}
impl SaveMessagePack for Bookmark {}
impl LoadYaml for Bookmark {}
impl LoadMessagePack for Bookmark {}
