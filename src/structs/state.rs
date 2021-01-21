use crate::ParseError;
use crate::{Map, Operator, Parsable, Value};

/// Typedef for state, which is a mapping of values.
pub type State = Map<String, Value>;

/// Trait to give state a `.update` function.
pub trait StateUpdatable {
    fn update(&mut self, state: &Self) -> Result<Self, ParseError>
    where
        Self: Sized;
}

impl StateUpdatable for State {
    /// Updates the state using a state modifier state_mod.
    /// Note that state_mod may NOT contain any keys not present in state.
    /// It's also assumed that all keys in state mod have been validated.
    fn update(&mut self, state: &Self) -> Result<Self, ParseError> {
        let mut root_vars = Self::new();
        for (key, value) in state {
            let statemod = StateMod::parse(key)?;
            if self.contains_key(statemod.var) {
                statemod.apply(self, value);
            } else {
                root_vars.insert(key.clone(), value.clone());
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

impl<'a> Parsable<'a> for StateMod<'a> {
    fn parse(text: &'a str) -> Result<Self, ParseError> {
        let split: Vec<&str> = text.split(' ').collect();
        if split.len() != 2 {
            return Err(perror!(
                "State modification must be of the form 'VAR [+-=]:'."
            ));
        }
        Ok(Self {
            var: split[0],
            op: Operator::parse(split[1])?,
        })
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
