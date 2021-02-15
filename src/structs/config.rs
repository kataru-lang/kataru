use super::{Map, Params, State};
use crate::error::ParseError;
use crate::traits::{FromYaml, Merge};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    pub state: State,
    #[serde(default)]
    pub cmds: Map<String, Option<Params>>,
    #[serde(default)]
    pub characters: Map<String, CharacterData>,
}

impl FromYaml<'_> for Config {}

impl Merge for Config {
    fn merge(&mut self, other: &mut Self) -> Result<(), ParseError> {
        self.characters.merge(&mut other.characters)?;
        self.cmds.merge(&mut other.cmds)?;
        self.state.merge(&mut other.state)?;
        Ok(())
    }
}
