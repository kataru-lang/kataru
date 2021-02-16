use crate::error::Error;
use crate::traits::FromStr;

#[derive(Debug)]
pub enum Operator {
    ADD,
    SUB,
    SET,
}

impl FromStr<'_> for Operator {
    fn from_str(op: &str) -> Result<Self, Error> {
        match op {
            "+" => Ok(Self::ADD),
            "-" => Ok(Self::SUB),
            _ => Err(error!("No valid Operator matches {}", op)),
        }
    }
}
