use super::{Map, State, Value};
use crate::error::ParseError;
use crate::traits::{Mergeable, Parsable};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    pub description: String,
}

pub type Characters = Map<String, CharacterData>;
pub type Params = Map<String, Value>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    pub state: State,
    #[serde(default)]
    pub cmds: Map<String, Params>,
    #[serde(default)]
    pub characters: Characters,
}

impl Parsable<'_> for Config {
    fn parse(text: &str) -> Result<Self, ParseError> {
        match serde_yaml::from_str(text) {
            Ok(config) => Ok(config),
            Err(e) => Err(perror!("{}", e)),
        }
    }
}

impl Mergeable for Config {
    fn merge(&mut self, other: &mut Self) -> Result<(), ParseError> {
        self.characters.merge(&mut other.characters)?;
        self.cmds.merge(&mut other.cmds)?;
        self.state.merge(&mut other.state)?;
        Ok(())
    }
}
