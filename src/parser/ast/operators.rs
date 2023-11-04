use crate::{parser::Rule, Error, Result, TryFrom};

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    Neq,
    Leq,
    Lt,
    Geq,
    Gt,
}
impl TryFrom<Rule> for BinaryOperator {
    fn try_from(rule: Rule) -> Result<Self> {
        Ok(match rule {
            Rule::Add => Self::Add,
            Rule::Sub => Self::Sub,
            Rule::Mul => Self::Mul,
            Rule::Div => Self::Div,
            Rule::And => Self::And,
            Rule::Or => Self::Or,
            Rule::Eq => Self::Eq,
            Rule::Neq => Self::Neq,
            Rule::Leq => Self::Leq,
            Rule::Lt => Self::Lt,
            Rule::Geq => Self::Geq,
            Rule::Gt => Self::Gt,
            _ => return Err(error!("Invalid BinaryOperator")),
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Add,
    Sub,
    Not,
}
impl TryFrom<Rule> for UnaryOperator {
    fn try_from(rule: Rule) -> Result<Self> {
        Ok(match rule {
            Rule::Add => Self::Add,
            Rule::Sub => Self::Sub,
            Rule::Not => Self::Not,
            _ => return Err(error!("Invalid UnaryOperator")),
        })
    }
}
