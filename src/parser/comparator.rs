use super::Parsable;
use crate::ValidationError;

#[derive(Debug, PartialEq)]
pub enum Comparator {
    EQ,
    NEQ,
    GT,
    GEQ,
    LT,
    LEQ,
}

impl Parsable<'_> for Comparator {
    fn parse(op: &str) -> Result<Self, ValidationError> {
        match op {
            "==" => Ok(Self::EQ),
            "!=" => Ok(Self::NEQ),
            ">" => Ok(Self::GT),
            ">=" => Ok(Self::GEQ),
            "<" => Ok(Self::LT),
            "<=" => Ok(Self::LEQ),
            _ => Err(verror!("No valid comparator matches {}", op)),
        }
    }
}
