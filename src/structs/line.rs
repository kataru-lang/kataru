use super::{Branches, Choices, Map, State, Value};
use serde::{Deserialize, Serialize};

pub type Dialogue = Map<String, String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cmd {
    pub cmd: String,
    #[serde(default)]
    pub params: Map<String, Value>,
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
    Cmd(Cmd),
    Dialogue(Dialogue),
    Continue,
    Break,
    InvalidChoice,
    End,
    Error,
}
