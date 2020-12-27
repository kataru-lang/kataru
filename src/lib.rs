#[macro_use]
extern crate lazy_static;

#[macro_use]
mod error;
mod parser;
mod runner;
mod structs;
mod validator;
mod vars;

pub use error::ValidationError;
pub use parser::{Parsable, State, Value};
pub use runner::Runner;
pub use structs::{Config, Dialogue, Line, Passage, Story};
pub use validator::validate;
