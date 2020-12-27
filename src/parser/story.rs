use super::{Branches, Map, Parsable, State};
use crate::ValidationError;
use serde::{Deserialize, Serialize};

pub type Dialogue = Map<String, String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choices {
    pub choices: Map<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goto {
    pub goto: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetCmd {
    pub set: State,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Line {
    Branches(Branches),
    Choices(Choices),
    Goto(Goto),
    Text(String),
    SetCmd(SetCmd),
    Dialogue(Dialogue),
    Continue,
    Break,
    InvalidChoice,
}

pub type Passage = Vec<Line>;

pub type Story = Map<String, Passage>;

impl Parsable<'_> for Story {
    fn parse(text: &str) -> Result<Self, ValidationError> {
        match serde_yaml::from_str(text) {
            Ok(story) => Ok(story),
            Err(e) => Err(verror!("{}", e)),
        }
    }
}
