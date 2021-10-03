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
    #[serde(default)]
    pub timeout: f64,
    pub input: Map<String, String>,
}

/// Internal representation of a line used for deserializing YAML.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawLine {
    Branches(Branches),
    SetCommand(SetCommand),
    Input(Input),
    Choices(RawChoices),
    Command(RawCommand),
    PositionalCommand(PositionalCommand),
    Call(Call),
    Return(Return),
    Text(String),
    Dialogue(Map<String, String>),
}

/// Public interface for a line in a Kataru script.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Line {
    Choices(Choices),
    InvalidChoice,
    Dialogue(Dialogue),
    Input(Input),
    Command(Command),
    End,
}

/// All lines take up 1 except for branches,
/// which need their length recursively computed.
pub fn line_len(lines: &[RawLine]) -> usize {
    let mut length = 0;
    for line in lines {
        match &line {
            RawLine::Branches(branches) => length += branches.line_len(),
            RawLine::Choices(choices) => length += choices.line_len(),
            _ => length += 1,
        }
    }
    length
}
