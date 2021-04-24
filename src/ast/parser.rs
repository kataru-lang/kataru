use pest::{self, iterators::Pair, Parser};

use super::{BinaryExpr, Node, UnaryExpr};
use crate::{traits::FromStr, Bookmark, Error, Operator, Result, Value};

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
            ("1", vec![Node::Value(Value::Number(1.))]),
            ("-2", vec![Node::Value(Value::Number(-2.))]),
            (
                "not true",
                vec![Node::UnaryExpr(UnaryExpr {
                    op: Operator::Not,
                    child: Box::new(Node::Value(Value::Bool(true))),
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
