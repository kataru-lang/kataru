#[macro_use]
extern crate lazy_static;

#[macro_use]
mod packer;
mod runner;
mod vars;

pub use kataru_parser::{validate, Config, Dialogue, Line, Parsable, Passage, State, Story, Value};
pub use packer::pack;
pub use runner::Runner;
