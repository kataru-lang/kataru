use super::{AssignOperator, Map};
use crate::{
    error::{Error, Result},
    traits::FromStr,
    Value,
};

/// Typedef for state, which is a mapping of values.
pub type State = Map<String, Value>;

#[derive(Debug)]
pub struct StateMod<'a> {
    pub var: &'a str,
    pub op: AssignOperator,
}

impl<'a> FromStr<'a> for StateMod<'a> {
    fn from_str(text: &'a str) -> Result<Self> {
        let split: Vec<&str> = text.split(' ').collect();
        if split.len() == 1 {
            return Ok(Self {
                var: &split[0][1..],
                op: AssignOperator::None,
            });
        } else if split.len() == 2 {
            return Ok(Self {
                var: &split[0][1..],
                op: AssignOperator::from_str(split[1])?,
            });
        }
        Err(error!(
            "State modification must be of the form 'VAR [+-]:'."
        ))
    }
}

impl<'a> StateMod<'a> {
    pub fn apply(&self, state: &mut State, value: Value) -> Result<()> {
        if let Some(state_value) = state.get_mut(self.var) {
            match self.op {
                AssignOperator::None => *state_value = value.clone(),
                AssignOperator::Add => *state_value += value,
                AssignOperator::Sub => *state_value -= value,
            };
            Ok(())
        } else {
            Err(error!("Undefined state variable '{}'", self.var))
        }
    }
}
