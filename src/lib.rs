#[macro_use]
extern crate lazy_static;

#[macro_use]
mod error;
mod parser;
mod runner;
mod validator;
mod vars;

pub use error::ValidationError;
pub use parser::{Config, Dialogue, Line, Parsable, Passage, State, Story, Value};
pub use runner::Runner;
pub use validator::validate;
