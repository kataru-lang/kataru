mod branches;
mod comparator;
mod conditional;
mod operator;
mod state;
mod value;

use crate::ValidationError;
pub use branches::{Branchable, Branches};
pub use comparator::Comparator;
pub use conditional::Conditional;
pub use operator::Operator;
pub use state::{State, StateMod, StateUpdatable};
pub use value::Value;

pub trait Parsable<'a> {
    fn parse(text: &'a str) -> Result<Self, ValidationError>
    where
        Self: std::marker::Sized;
}
