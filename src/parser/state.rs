use super::{Operator, Parsable, Value};
use crate::structs::Map;
use crate::ValidationError;

pub trait StateUpdatable {
    fn update(&mut self, other: &Self) -> Result<(), ValidationError>;
}

pub type State = Map<String, Value>;

#[derive(Debug)]
pub struct StateMod<'a> {
    pub var: &'a str,
    pub op: Operator,
}
impl<'a> Parsable<'a> for StateMod<'a> {
    fn parse(text: &'a str) -> Result<Self, ValidationError> {
        let split: Vec<&str> = text.split(' ').collect();
        if split.len() != 2 {
            return Err(verror!(
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

impl StateUpdatable for State {
    /// Updates the state using a state modifier state_mod.
    /// Note that state_mod may NOT contain any keys not present in state.
    /// It's also assumed that all keys in state mod have been validated.
    fn update(&mut self, state_mod: &Self) -> Result<(), ValidationError> {
        for (key, value) in state_mod {
            StateMod::parse(key)?.apply(self, value);
        }
        Ok(())
    }
}
