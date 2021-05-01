use super::{Map, QualifiedName, State, Story};
use crate::{
    error::{Error, Result},
    traits::FromStr,
    traits::{FromMessagePack, FromYaml, LoadYaml, SaveMessagePack},
    Load, LoadMessagePack, Save, SaveYaml, StateMod, Value, GLOBAL,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Position {
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    pub passage: String,
    #[serde(default)]
    pub line: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            namespace: GLOBAL.to_string(),
            passage: String::new(),
            line: 0,
        }
    }
}

/// All data necessary to find your place in the story.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct Bookmark {
    #[serde(default)]
    pub state: Map<String, State>,
    #[serde(default)]
    position: Position,
    #[serde(default)]
    pub stack: Vec<Position>,
    #[serde(default)]
    pub snapshots: Map<String, Vec<Position>>,
}

impl<'a> Bookmark {
    pub fn new(state: Map<String, State>) -> Self {
        Self {
            state,
            ..Self::default()
        }
    }

    #[inline]
    pub fn namespace(&self) -> &str {
        &self.position.namespace
    }

    #[inline]
    pub fn passage(&self) -> &str {
        &self.position.passage
    }

    #[inline]
    pub fn line(&self) -> usize {
        self.position.line
    }

    #[inline]
    pub fn next_line(&mut self) {
        self.position.line += 1
    }

    #[inline]
    pub fn skip_lines(&mut self, lines: usize) {
        self.position.line += lines
    }

    #[inline]
    pub fn set_line(&mut self, line: usize) {
        self.position.line = line
    }

    #[inline]
    pub fn position(&self) -> &Position {
        &self.position
    }

    #[inline]
    pub fn set_passage(&mut self, passage: String) {
        self.position.passage = passage;
    }

    #[inline]
    pub fn set_namespace(&mut self, namespace: String) {
        self.position.namespace = namespace;
    }

    #[inline]
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn update_position(&mut self, qname: QualifiedName) {
        self.position.namespace = qname.namespace;
        self.position.passage = qname.name;
    }

    /// Gets the value for a given variable.
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
            return Err(error!("No state for global namespace"));
        }

        // Return error if there is no passage name in either namespace.
        Err(error!(
            "Variable '{}' could not be found in '{}' nor global namespace state",
            qname.name, qname.namespace
        ))
    }

    /// Returns mutable state.
    pub fn state(&'a mut self) -> Result<&'a mut State> {
        match self.state.get_mut(&self.position.namespace) {
            Some(state) => Ok(state),
            None => Err(error!("Invalid namespace {}", &self.position.namespace)),
        }
    }

    /// Returns mutable global state.
    pub fn global_state(&'a mut self) -> Result<&'a mut State> {
        match self.state.get_mut(GLOBAL) {
            Some(state) => Ok(state),
            None => Err(error!("No global namespace")),
        }
    }

    /// Given a mapping of state changes, updates the bookmark's state.
    /// `passage` is required for $passage variables.
    pub fn set_state(&mut self, state: &State) -> Result<()> {
        for (key, value) in state {
            // If a expression, evaluate. TODO: avoid clone.
            let mut value = value.clone();
            value.eval_as_expr(self)?;

            // If contains ${passage} expansion, text should refer to the replaced text.
            // Otherwise it should simply be the key.
            let replaced: String;
            let mut text = key;
            if key.starts_with("$passage") {
                replaced = format!("${}{}", &self.position.passage, &text["$passage".len()..]);
                text = &replaced;
            }

            let statemod = StateMod::from_str(text)?;
            println!("statemod: {:?}", statemod);
            let local_state = self.state()?;
            if local_state.contains_key(statemod.var) {
                statemod.apply(local_state, value);
            } else {
                let global_state = self.global_state()?;
                statemod.apply(global_state, value);
            }
        }
        Ok(())
    }

    /// Updates `state[var] = val` iff `var` not already in `state`.
    fn default_val(state: &mut State, var: &str, val: &Value) {
        if state.get(var).is_none() {
            state.insert(var.to_string(), val.clone());
        }
    }

    /// Defaults bookmark state based on the story.
    pub fn init_state(&mut self, story: &Story) {
        for (namespace, section) in story {
            if self.state.get(namespace).is_none() {
                self.state.insert(namespace.to_string(), State::default());
            }

            let namespace_state = self.state.get_mut(namespace).unwrap();
            for (var, val) in section.state() {
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

    /// Saves a snapshot of the stack under `name`.
    pub fn save_snapshot(&mut self, name: &str) {
        let mut stack = self.stack.clone();
        stack.push(self.position.clone());
        self.snapshots.insert(name.to_string(), stack);
    }

    /// Loads a snapshot of the stack under `name`.
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

    /// Returns true if a character is local.
    pub fn character_is_local(&self, story: &Story, character: &str) -> bool {
        if self.namespace() == GLOBAL {
            return false;
        }

        // The character is local if the section exists and the character
        // is defined in the section.
        if let Some(section) = story.get(self.namespace()) {
            if section.has_character(character) {
                return true;
            }
        }
        false
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
