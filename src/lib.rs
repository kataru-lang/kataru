#[macro_use]
extern crate lazy_static;

#[macro_use]
mod runner;
mod vars;

pub use kataru_parser::{
    pack, validate, Config, Deserializable, Dialogue, Line, Parsable, Passage, State, Story, Value,
};
pub use runner::Runner;
