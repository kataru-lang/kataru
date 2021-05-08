use crate::structs::RawLine;
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
    None,
}

impl LineTag {
    pub fn tag(line_opt: &Option<&RawLine>) -> Self {
        match line_opt {
            Some(line) => match line {
                RawLine::Choices(_) => LineTag::Choices,
                RawLine::Dialogue(_) => LineTag::Dialogue,
                RawLine::Command(_) => LineTag::Command,
                RawLine::Input(_) => LineTag::Input,
                RawLine::InvalidChoice => LineTag::InvalidChoice,
                _ => LineTag::None,
            },
            None => LineTag::None,
        }
    }
}
