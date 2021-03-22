use super::{Branches, Choices, Command, Dialogue, Map, PositionalCommand, RawChoices, State};
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Line {
    Branches(Branches),
    Input(Input),
    SetCommand(SetCommand),
    RawChoices(RawChoices),
    Choices(Choices),
    Command(Command),
    PositionalCommand(PositionalCommand),
    Call(Call),
    Return(Return),
    Text(String),
    RawDialogue(Map<String, String>),
    Dialogue(Dialogue),
    Continue,
    Break,
    InvalidChoice,
    End,
}
