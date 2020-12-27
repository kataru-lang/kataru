use super::{Map, Parsable, State};
use crate::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterData {
    pub description: String,
}

pub type Characters = Map<String, CharacterData>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub passage: String,
    pub line: usize,
    pub state: State,
    pub characters: Characters,
}

impl Parsable<'_> for Config {
    fn parse(text: &str) -> Result<Self, ValidationError> {
        match serde_yaml::from_str(text) {
            Ok(config) => Ok(config),
            Err(e) => Err(verror!("{}", e)),
        }
    }
}
