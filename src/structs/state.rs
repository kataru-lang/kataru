use crate::error::{Error, Result};
use crate::traits::FromStr;
use crate::{Map, Operator, Value};

/// Typedef for state, which is a mapping of values.
pub type State = Map<String, Value>;

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
