use crate::error::Error;
use crate::traits::FromStr;

#[derive(Debug, PartialEq)]
pub enum Comparator {
    EQ,
    NEQ,
    GT,
    GEQ,
    LT,
    LEQ,
}

impl FromStr<'_> for Comparator {
    fn from_str(op: &str) -> Result<Self, Error> {
        match op {
            "==" => Ok(Self::EQ),
            "!=" => Ok(Self::NEQ),
            ">" => Ok(Self::GT),
            ">=" => Ok(Self::GEQ),
            "<" => Ok(Self::LT),
            "<=" => Ok(Self::LEQ),
            _ => Err(error!("No valid comparator matches {}", op)),
        }
    }
}
