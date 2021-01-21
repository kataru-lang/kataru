use crate::error::ParseError;
use crate::traits::Parsable;

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
    fn parse(op: &str) -> Result<Self, ParseError> {
        match op {
            "==" => Ok(Self::EQ),
            "!=" => Ok(Self::NEQ),
            ">" => Ok(Self::GT),
            ">=" => Ok(Self::GEQ),
            "<" => Ok(Self::LT),
            "<=" => Ok(Self::LEQ),
            _ => Err(perror!("No valid comparator matches {}", op)),
        }
    }
}
