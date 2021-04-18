use super::{Operator, Value};
use crate::{Bookmark, Result};
use std::fmt;

mod eval;
mod parser;

pub use parser::IntoAST;

/// AST `Node` created from pest pairs by `parser`.
#[derive(Debug, PartialEq)]
pub enum Node {
    Value(Value),
    UnaryExpr(UnaryExpr),
    BinaryExpr(BinaryExpr),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self {
            Node::Value(n) => write!(f, "{}", n),
            Node::UnaryExpr(expr) => write!(f, "{}", expr),
            Node::BinaryExpr(expr) => write!(f, "{}", expr),
        }
    }
}

/// Expression with a single operand.
#[derive(Debug, PartialEq)]
pub struct UnaryExpr {
    op: Operator,
    child: Box<Node>,
}

impl fmt::Display for UnaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}{}", self.op, self.child)
    }
}

/// Expression with two operands.
#[derive(Debug, PartialEq)]
pub struct BinaryExpr {
    op: Operator,
    lhs: Box<Node>,
    rhs: Box<Node>,
}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast() {}
}
