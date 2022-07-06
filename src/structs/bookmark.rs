use super::{Map, QualifiedName, State, Story};
use crate::{
    error::{Error, Result},
    traits::FromStr,
    traits::{FromMessagePack, FromYaml, LoadYaml, SaveMessagePack},
    Load, LoadMessagePack, Save, SaveYaml, Section, StateMod, Value, GLOBAL,
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

    pub fn update_position(&mut self, namespace: String, passage: String) {
        self.position.namespace = namespace;
        self.position.passage = passage;
    }

    /// Gets the value for a given variable.
    pub fn value(&'a self, var: &str) -> Result<&'a Value> {
        let qname = QualifiedName::from(&self.position.namespace, var);
        for namespace in qname.resolve() {
            if let Some(section) = self.state.get(namespace) {
                if let Some(val) = section.get(qname.name) {
                    return Ok(val);
                }
            } else {
                return Err(error!("No state for namespace '{}'", namespace));
            }
        }
        Err(error!(
            "Var '{}' could not be found in namespace '{}' nor any of its parents.",
            qname.name, qname.namespace
        ))
    }

    /// Gets the value for a given variable.
    pub fn set_value(&'a mut self, statemod: StateMod, value: Value) -> Result<()> {
        let qname = QualifiedName::from(&self.position.namespace, statemod.var);
        for namespace in qname.resolve() {
            if let Some(section) = self.state.get_mut(namespace) {
                if let Some(value_mut) = section.get_mut(qname.name) {
                    return statemod.apply(value_mut, value);
                }
            } else {
                return Err(error!("No state for namespace '{}'", namespace));
            }
        }
        Err(error!(
            "Var '{}' could not be found in namespace '{}' nor any of its parents.",
            qname.name, qname.namespace
        ))
    }

    /// Given a mapping of state changes, updates the bookmark's state.
    pub fn set_state(&mut self, state: &State) -> Result<()> {
        for (key, value) in state {
            // If a expression, evaluate. TODO: avoid clone.
            let mut value = value.clone();
            value.eval_as_expr(self)?;

            // If contains ${passage} expansion, text should refer to the replaced text.
            // Otherwise it should simply be the key.
            let replaced: String;
            let mut statemod_expr = key;
            if key.starts_with("$passage") {
                replaced = format!(
                    "${}{}",
                    &self.position.passage,
                    &statemod_expr["$passage".len()..]
                );
                statemod_expr = &replaced;
            }

            // Parse the statemod expression and update state accordingly.
            self.set_value(StateMod::from_str(statemod_expr)?, value)?;
        }
        Ok(())
    }

    /// Updates `state[var] = val` iff `var` not already in `state`.
    fn default_val(state: &mut State, var: &str, val: &Value) {
        if state.get(var).is_none() {
            state.insert(var.to_string(), val.clone());
        }
    }

    fn default_passage_expansion(
        var: &str,
        val: &Value,
        section: &Section,
        section_state: &mut State,
    ) {
        for passage in section.passages.keys() {
            let replaced = format!("{}{}", passage, &var["$passage".len()..]);
            Self::default_val(section_state, &replaced, val);
        }
    }

    /// Defaults this section's state.
    fn init_section_state(&mut self, namespace: &str, story: &Story, section: &Section) {
        let section_state = match self.state.get_mut(namespace) {
            None => {
                self.state.insert(namespace.to_string(), State::default());
                self.state.get_mut(namespace).unwrap()
            }
            Some(state) => state,
        };
        for (var, val) in section.state() {
            if var.starts_with("$passage") {
                Self::default_passage_expansion(var, val, section, section_state);
            } else {
                Self::default_val(section_state, &var, &val);
            }
        }
        Self::init_parent_expansions(namespace, story, section, section_state)
    }

    /// For all parents' expansion variables, defaults values for the entities in this section.
    fn init_parent_expansions(
        namespace: &str,
        story: &Story,
        section: &Section,
        section_state: &mut State,
    ) {
        let qname = QualifiedName::from(namespace, "");
        let mut parents_iter = qname.resolve();
        parents_iter.next(); // Don't check this section, only check parents.
        for parent_namespace in parents_iter {
            if let Some(parent_section) = story.sections.get(parent_namespace) {
                for (var, val) in parent_section.state() {
                    if var.starts_with("$passage") {
                        Self::default_passage_expansion(var, val, section, section_state);
                    }
                }
            }
        }
    }

    /// Defaults bookmark state based on the story.
    pub fn init_state(&mut self, story: &Story) {
        for (namespace, section) in &story.sections {
            self.init_section_state(namespace, story, section);
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

    /// Gets the qualified character name (prefixed with namespace if not global).
    pub fn qualified_character_name(&self, story: &Story, character: &str) -> Result<String> {
        let qname = QualifiedName::from(self.namespace(), character);
        let (resolved_namespace, _section, _chardata) = story.character(&qname)?;
        Ok(qname.to_string(resolved_namespace))
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
