use super::{Map, Params, State};
use crate::traits::{FromYaml, Merge};
use crate::{error::Error, SetCmd};
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
    pub commands: Map<String, Option<Params>>,
    #[serde(default)]
    pub characters: Map<String, Option<CharacterData>>,
    #[serde(default)]
    pub attributes: Map<String, Option<String>>,
    #[serde(default)]
    #[serde(rename = "onPassage")]
    pub on_passage: Option<SetCmd>,
}

impl FromYaml for Config {}

impl Merge for Config {
    fn merge(&mut self, other: &mut Self) -> Result<(), Error> {
        self.characters.merge(&mut other.characters)?;
        self.commands.merge(&mut other.commands)?;
        self.state.merge(&mut other.state)?;
        self.attributes.merge(&mut other.attributes)?;
        if self.on_passage.is_none() && other.on_passage.is_some() {
            self.on_passage = other.on_passage.clone();
        }
        Ok(())
    }
}
