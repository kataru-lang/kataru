use crate::error::Error;
use crate::traits::{FromStr, IntoStr};
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    And,
    Or,
    Not,

    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
}

impl Operator {
    const ADD: &'static str = "+";
    const SUB: &'static str = "-";
    const AND: &'static str = "and";
    const OR: &'static str = "or";
    const NOT: &'static str = "not";

    const EQ: &'static str = "==";
    const NEQ: &'static str = "!=";
    const LT: &'static str = "<";
    const LEQ: &'static str = "<=";
    const GT: &'static str = ">";
    const GEQ: &'static str = ">=";
}

impl IntoStr for Operator {
    fn into_str(&self) -> &str {
        match *self {
            Self::Add => Self::ADD,
            Self::Sub => Self::SUB,
            Self::And => Self::AND,
            Self::Or => Self::OR,
            Self::Not => Self::NOT,

            Self::Eq => Self::EQ,
            Self::Neq => Self::NEQ,
            Self::Lt => Self::LT,
            Self::Leq => Self::LEQ,
            Self::Gt => Self::GT,
            Self::Geq => Self::GEQ,
        }
    }
}

impl FromStr<'_> for Operator {
    fn from_str(text: &str) -> Result<Self, Error> {
        let op = match text {
            Self::ADD => Self::Add,
            Self::SUB => Self::Sub,
            Self::AND => Self::And,
            Self::OR => Self::Or,
            Self::NOT => Self::Not,

            Self::EQ => Self::Eq,
            Self::NEQ => Self::Neq,
            Self::LT => Self::Lt,
            Self::LEQ => Self::Leq,
            Self::GT => Self::Gt,
            Self::GEQ => Self::Geq,
            _ => return Err(error!("No valid Operator matches {}", text)),
        };
        Ok(op)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.into_str())
    }
}

#[derive(Debug, PartialEq)]
pub enum AssignOperator {
    None,
    Add,
    Sub,
}

impl AssignOperator {
    const ADD: &'static str = "+";
    const SUB: &'static str = "-";
}

impl IntoStr for AssignOperator {
    fn into_str(&self) -> &str {
        match *self {
            Self::Add => Self::ADD,
            Self::Sub => Self::SUB,
            Self::None => "",
        }
    }
}

impl FromStr<'_> for AssignOperator {
    fn from_str(text: &str) -> Result<Self, Error> {
        let op = match text {
            Self::SUB => Self::Sub,
            Self::ADD => Self::Add,
            _ => return Err(error!("No valid assignment operator matches {}", text)),
        };
        Ok(op)
    }
}
