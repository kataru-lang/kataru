use super::attributes::AttributeConfig;
use super::{Map, Params, State};
use crate::traits::{FromYaml, Merge};
use crate::{error::Error, SetCommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
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
    pub attributes: Map<String, Option<AttributeConfig>>,
    #[serde(default)]
    #[serde(rename = "onEnter")]
    pub on_enter: Option<SetCommand>,
    #[serde(default)]
    #[serde(rename = "onExit")]
    pub on_exit: Option<SetCommand>,
}

impl FromYaml for Config {}

impl Merge for Config {
    fn merge(&mut self, other: &mut Self) -> Result<(), Error> {
        self.characters.merge(&mut other.characters)?;
        self.commands.merge(&mut other.commands)?;
        self.state.merge(&mut other.state)?;
        self.attributes.merge(&mut other.attributes)?;

        // Merge automatic setters.
        if self.on_enter.is_none() && other.on_enter.is_some() {
            self.on_enter = other.on_enter.clone();
        }
        if self.on_exit.is_none() && other.on_exit.is_some() {
            self.on_exit = other.on_exit.clone();
        }
        Ok(())
    }
}
