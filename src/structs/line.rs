use super::{Branches, Choices, Dialogue, Map, State, Value};
use serde::{Deserialize, Serialize};

pub type Params = Map<String, Value>;
pub type Cmd = Map<String, Params>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goto {
    pub goto: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetCmd {
    pub set: State,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputCmd {
    pub input: Map<String, String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Line {
    Branches(Branches),
    Input(InputCmd),
    SetCmd(SetCmd),
    Commands(Vec<Cmd>),
    Choices(Choices),
    Goto(Goto),
    Text(String),
    _Dialogue(Map<String, String>),
    Dialogue(Dialogue),
    Continue,
    Break,
    InvalidChoice,
    End,
}
