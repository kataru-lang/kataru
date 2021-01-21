use super::{Branches, Map, State, Value};
use serde::{Deserialize, Serialize};

pub type Dialogue = Map<String, String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choices {
    pub choices: Map<String, String>,
    pub timeout: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cmd {
    pub cmd: String,
    pub params: Option<Map<String, Value>>,
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
}
