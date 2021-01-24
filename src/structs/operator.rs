use crate::error::ParseError;
use crate::traits::Parsable;

#[derive(Debug)]
pub enum Operator {
    ADD,
    SUB,
    SET,
}

impl Parsable<'_> for Operator {
    fn parse(op: &str) -> Result<Self, ParseError> {
        match op {
            "+" => Ok(Self::ADD),
            "-" => Ok(Self::SUB),
            _ => Err(perror!("No valid Operator matches {}", op)),
        }
    }
}
