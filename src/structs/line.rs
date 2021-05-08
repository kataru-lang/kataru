use super::{
    Branches, Choices, Command, Dialogue, Map, PositionalCommand, RawChoices, RawCommand, State,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Call {
    #[serde(rename = "call")]
    pub passage: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Return {
    pub r#return: (),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetCommand {
    pub set: State,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub input: Map<String, String>,
}

/// Internal representation of a line used for deserializing YAML.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawLine {
    Branches(Branches),
    Input(Input),
    SetCommand(SetCommand),
    RawChoices(RawChoices),
    #[serde(skip_serializing)]
    Choices(Choices),
    RawCommand(RawCommand),
    #[serde(skip_serializing)]
    Command(Command),
    PositionalCommand(PositionalCommand),
    Call(Call),
    Return(Return),
    Text(String),
    RawDialogue(Map<String, String>),
    #[serde(skip_serializing)]
    Dialogue(Dialogue),
    Continue,
    Break,
    InvalidChoice,
    End,
}

/// Public interface for a line in a Kataru script.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Line {
    Choices(Choices),
    InvalidChoice,
    Dialogue(Dialogue),
    Input(Input),
    Command(Command),
    None,
}

impl From<RawLine> for Line {
    fn from(raw: RawLine) -> Self {
        match raw {
            RawLine::Choices(choices) => Line::Choices(choices),
            RawLine::Dialogue(dialogue) => Line::Dialogue(dialogue),
            RawLine::Input(input) => Line::Input(input),
            RawLine::Command(command) => Line::Command(command),
            _ => Line::None,
        }
    }
}
