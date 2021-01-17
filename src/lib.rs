#[macro_use]
extern crate lazy_static;

#[macro_use]
mod runner;
mod vars;

pub use kataru_parser::{
    pack, validate, Choices, Cmd, Config, Deserializable, Dialogue, Goto, Line, Parsable, Passage,
    SetCmd, State, Story, Value,
};
pub use runner::Runner;
