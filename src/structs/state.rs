use crate::error::{Error, Result};
use crate::traits::FromStr;
use crate::{Map, Operator, Value};

/// Typedef for state, which is a mapping of values.
pub type State = Map<String, Value>;

/// Trait to give state a `.update` function.
pub trait StateUpdatable {
    fn update(&mut self, state: &Self, passage: &str) -> Result<Self>
    where
        Self: Sized;
}

impl StateUpdatable for State {
    /// Updates the state using a state modifier state_mod.
    /// Note that state_mod may NOT contain any keys not present in state.
    /// It's also assumed that all keys in state mod have been validated.
    fn update(&mut self, state: &Self, passage: &str) -> Result<Self> {
        let mut root_vars = Self::new();
        for (key, value) in state {
            // If contains ${passage} expansion, text should refer to the replaced text.
            // Otherwise it should simply be the key.
            #[allow(unused_assignments)]
            let mut replaced = String::new();
            let mut text = key;
            if key.starts_with("$passage") {
                replaced = format!("${}{}", passage, &text["$passage".len()..]);
                text = &replaced;
            }

            let statemod = StateMod::from_str(text)?;
            if self.contains_key(statemod.var) {
                statemod.apply(self, value);
            } else {
                root_vars.insert(text.clone(), value.clone());
            }
        }
        Ok(root_vars)
    }
}

#[derive(Debug)]
pub struct StateMod<'a> {
    pub var: &'a str,
    pub op: Operator,
}

impl<'a> FromStr<'a> for StateMod<'a> {
    fn from_str(text: &'a str) -> Result<Self> {
        let split: Vec<&str> = text.split(' ').collect();
        if split.len() == 1 {
            return Ok(Self {
                var: &split[0][1..],
                op: Operator::SET,
            });
        } else if split.len() == 2 {
            return Ok(Self {
                var: &split[0][1..],
                op: Operator::from_str(split[1])?,
            });
        }
        Err(error!(
            "State modification must be of the form 'VAR [+-]:'."
        ))
    }
}

impl<'a> StateMod<'a> {
    pub fn apply(&self, state: &mut State, value: &Value) {
        let state_value = state.get_mut(self.var).unwrap();
        match self.op {
            Operator::SET => *state_value = value.clone(),
            Operator::ADD => *state_value += value,
            Operator::SUB => *state_value -= value,
        }
    }
}
