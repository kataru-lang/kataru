#[macro_use]
extern crate lazy_static;

#[macro_use]
mod error;

#[macro_use]
mod runner;
mod packer;
mod structs;
mod traits;
mod validator;
mod vars;

pub use error::ParseError;
pub use packer::pack;
pub use structs::{
    Bookmark, Branchable, Branches, CharacterData, Choice, Choices, Cmd, Comparator, Conditional,
    Config, Dialogue, Goto, InputCmd, Line, Map, Operator, Params, Passage, Passages, Section,
    SetCmd, State, StateMod, StateUpdatable, Story, StoryGetters, Value,
};
pub use traits::{Deserializable, Loadable, Mergeable, Parsable};
pub use validator::Validator;

pub use runner::Runner;
