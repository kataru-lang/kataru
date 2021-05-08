use crate::Line;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineTag {
    Choices,
    InvalidChoice,
    Dialogue,
    Input,
    Command,
    End,
}

impl LineTag {
    pub fn tag(line: &Line) -> Self {
        match line {
            Line::Choices(_) => LineTag::Choices,
            Line::Dialogue(_) => LineTag::Dialogue,
            Line::Command(_) => LineTag::Command,
            Line::Input(_) => LineTag::Input,
            Line::InvalidChoice => LineTag::InvalidChoice,
            Line::End => LineTag::End,
        }
    }
}
