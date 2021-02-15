use crate::error::ParseError;
use crate::traits::FromStr;

#[derive(Debug)]
pub enum Operator {
    ADD,
    SUB,
    SET,
}

impl FromStr<'_> for Operator {
    fn from_str(op: &str) -> Result<Self, ParseError> {
        match op {
            "+" => Ok(Self::ADD),
            "-" => Ok(Self::SUB),
            _ => Err(perror!("No valid Operator matches {}", op)),
        }
    }
}
