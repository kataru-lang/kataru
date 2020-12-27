mod branches;
mod comparator;
mod conditional;
mod config;
mod operator;
mod state;
mod story;
mod value;

use crate::ValidationError;
pub use branches::{Branchable, Branches};
pub use comparator::Comparator;
pub use conditional::Conditional;
pub use config::Config;
pub use operator::Operator;
pub use state::{State, StateMod, StateUpdatable};
use std::collections::BTreeMap;
pub use story::{Choices, Dialogue, Line, Passage, Story};
pub use value::Value;

pub trait Parsable<'a> {
    fn parse(text: &'a str) -> Result<Self, ValidationError>
    where
        Self: std::marker::Sized;
}

pub type Map<K, V> = BTreeMap<K, V>;
