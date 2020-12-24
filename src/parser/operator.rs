use super::Parsable;
use crate::ValidationError;

#[derive(Debug)]
pub enum Operator {
    ADD,
    SUB,
    SET,
}

impl Parsable<'_> for Operator {
    fn parse(op: &str) -> Result<Self, ValidationError> {
        match op {
            "+=" => Ok(Self::ADD),
            "-=" => Ok(Self::SUB),
            "=" => Ok(Self::SET),
            _ => Err(verror!("No valid Operator matches {}", op)),
        }
    }
}
