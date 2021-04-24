#[macro_use]
extern crate lazy_static;

#[cfg_attr(test, macro_use)]
extern crate maplit;

#[macro_use]
mod error;

#[macro_use]
mod runner;
mod packer;
mod structs;
mod tagger;
mod traits;
mod validator;
mod value;
mod vars;

pub use error::{Error, Result};
pub use packer::pack;
pub use runner::Runner;
pub use structs::{
    AssignOperator, Bookmark, Branchable, Branches, CharacterData, Choices, Command, Config,
    Dialogue, Input, Line, Map, Operator, Params, Passage, Passages, Position, Section, SetCommand,
    State, StateMod, Story, StoryGetters, GLOBAL,
};
pub use tagger::LineTag;
pub use traits::{
    FromMessagePack, FromYaml, Load, LoadMessagePack, LoadYaml, Merge, Save, SaveMessagePack,
    SaveYaml,
};
pub use validator::Validator;
pub use value::Value;
pub use vars::{contains_var, extract_var};
