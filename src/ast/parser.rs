use pest::{self, iterators::Pair, Parser};

use super::{BinaryExpr, Node, UnaryExpr};
use crate::{traits::FromStr, Bookmark, Error, Operator, Result, Value};

/// Pest parser generated from ast/grammar.pest.
#[derive(pest_derive::Parser)]
#[grammar = "ast/grammar.pest"]
struct ExprParser;

/// Trait to convert a type into an Expr AST.
pub trait IntoAST {
    fn into_ast(&self, bookmark: &Bookmark) -> Result<Vec<Node>>;
}

impl IntoAST for str {
    fn into_ast(&self, bookmark: &Bookmark) -> Result<Vec<Node>> {
        let mut ast = Vec::new();
        let pairs = match ExprParser::parse(Rule::Program, self) {
            Ok(pairs) => pairs,
            Err(e) => return Err(error!("Pest parsing error: {:?}", e)),
        };

        for pair in pairs {
            if let Rule::Expr = pair.as_rule() {
                let node = pair.into_node(bookmark)?;
                ast.push(node);
            }
        }
        Ok(ast)
    }
}

/// Trait to create a type from a pest pair.
pub trait FromPair: Sized {
    fn from_pair(pair: Pair<Rule>, bookmark: &Bookmark) -> Result<Self>;
}

impl FromPair for Node {
    /// Recursively parse children to build AST.
    fn from_pair(pair: Pair<Rule>, bookmark: &Bookmark) -> Result<Self> {
        match pair.into_inner().next() {
            Some(child) => child.into_node(bookmark),
            None => Err(error!("Pest parsing error: node was None.")),
        }
    }
}

impl FromPair for UnaryExpr {
    /// Parse a unary expr from pest pair.
    fn from_pair(pair: Pair<Rule>, bookmark: &Bookmark) -> Result<Self> {
        let mut it = pair.into_inner();
        if let (Some(op), Some(child)) = (it.next(), it.next()) {
            Ok(Self {
                op: Operator::from_str(op.as_str())?,
                child: Box::new(child.into_node(bookmark)?),
            })
        } else {
            Err(error!("Pest parsing error: node was None."))
        }
    }
}

impl FromPair for BinaryExpr {
    /// Parse a binary expr from pest pair.
    fn from_pair(pair: Pair<Rule>, bookmark: &Bookmark) -> Result<Self> {
        let mut it = pair.into_inner();
        if let (Some(lhs), Some(op), Some(rhs)) = (it.next(), it.next(), it.next()) {
            Ok(Self {
                op: Operator::from_str(op.as_str())?,
                lhs: Box::new(lhs.into_node(bookmark)?),
                rhs: Box::new(rhs.into_node(bookmark)?),
            })
        } else {
            Err(error!("Pest parsing error: node was None."))
        }
    }
}

/// Trait to turn a type into an AST `Node`.
pub trait IntoNode {
    fn into_node(self, bookmark: &Bookmark) -> Result<Node>;
}

impl IntoNode for Pair<'_, Rule> {
    /// Turns a pest pair into an AST `Node`.
    fn into_node(self, bookmark: &Bookmark) -> Result<Node> {
        match self.as_rule() {
            Rule::Expr => Node::from_pair(self, bookmark),
            Rule::UnaryExpr => Ok(Node::UnaryExpr(UnaryExpr::from_pair(self, bookmark)?)),
            Rule::BinaryExpr => Ok(Node::BinaryExpr(BinaryExpr::from_pair(self, bookmark)?)),
            Rule::Value => Ok(Node::Value(Value::from_token(self.as_str(), bookmark)?)),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shared util test runner.
    fn run_tests(tests: Vec<(&str, Vec<Node>)>, bookmark: &Bookmark) {
        for (expr, expected) in tests {
            let ast = expr.into_ast(&bookmark).unwrap();
            assert_eq!(ast, expected);
            assert_eq!(format!("{}", ast[0]), expr);
        }
    }

    #[test]
    fn test_error() {
        let bookmark = Bookmark::default();
        assert!("b".into_ast(&bookmark).is_err());
    }

    #[test]
    fn test_unary_expr() {
        let bookmark = Bookmark::default();
        let tests = vec![
            (
                "+1",
                vec![Node::UnaryExpr(UnaryExpr {
                    op: Operator::Add,
                    child: Box::new(Node::Value(Value::Number(1.))),
                })],
            ),
            (
                "-2",
                vec![Node::UnaryExpr(UnaryExpr {
                    op: Operator::Sub,
                    child: Box::new(Node::Value(Value::Number(2.))),
                })],
            ),
        ];
        run_tests(tests, &bookmark);
    }

    #[test]
    fn test_binary_expr() {
        let bookmark = Bookmark::default();
        let tests = vec![
            (
                "1 + 2",
                vec![Node::BinaryExpr(BinaryExpr {
                    op: Operator::Add,
                    lhs: Box::new(Node::Value(Value::Number(1.))),
                    rhs: Box::new(Node::Value(Value::Number(2.))),
                })],
            ),
            (
                "1 - 2",
                vec![Node::BinaryExpr(BinaryExpr {
                    op: Operator::Sub,
                    lhs: Box::new(Node::Value(Value::Number(1.))),
                    rhs: Box::new(Node::Value(Value::Number(2.))),
                })],
            ),
        ];
        run_tests(tests, &bookmark);
    }

    #[test]
    fn test_nested_expr() {
        let bookmark = Bookmark::default();
        let tests = vec![
            ("1 + 2 + 3", "(1 + 2) + 3"),
            ("1 + 2 + 3", "1 + (2 + 3)"),
            ("1 + 2 + 3 + 4", "1 + (2 + (3 + 4))"),
            ("1 + 2 + 3 - 4", "(1 + 2) + (3 - 4)"),
        ];
        for (expr, nested_expr) in tests {
            assert_eq!(
                expr,
                nested_expr
                    .into_ast(&bookmark)
                    .unwrap()
                    .iter()
                    .fold(String::new(), |acc, arg| acc + &format!("{}", &arg))
            );
        }
    }
}
