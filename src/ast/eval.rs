use super::{BinaryExpr, Node, UnaryExpr};
use crate::{Bookmark, Result, Value};

pub trait Evaluate {
    fn eval(&self, bookmark: &Bookmark) -> Result<Value>;
}

impl Evaluate for UnaryExpr {
    fn eval(&self, bookmark: &Bookmark) -> Result<Value> {
        let mut child_value = self.child.eval(bookmark)?;
        child_value.apply(self.op);
        Ok(child_value)
    }
}

impl Evaluate for BinaryExpr {
    fn eval(&self, bookmark: &Bookmark) -> Result<Value> {
        let mut lhs_value = self.lhs.eval(bookmark)?;
        let rhs_value = self.rhs.eval(bookmark)?;
        lhs_value.combine(self.op, &rhs_value);
        Ok(lhs_value)
    }
}

impl Evaluate for Node {
    fn eval(&self, bookmark: &Bookmark) -> Result<Value> {
        match self {
            Node::BinaryExpr(expr) => expr.eval(bookmark),
            Node::UnaryExpr(expr) => expr.eval(bookmark),
            Node::Value(value) => {
                let mut value = value.clone();
                value.eval_in_place(bookmark)?;
                Ok(value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::IntoAST;

    /// Shared util test runner.
    fn run_tests(tests: Vec<(&str, Value)>, bookmark: &Bookmark) {
        for (expr, expected) in tests {
            let ast = expr.into_ast(&bookmark).unwrap();
            assert_eq!(ast.len(), 1);
            assert_eq!(ast[0].eval(bookmark).unwrap(), expected);
        }
    }

    #[test]
    fn test_eval() {
        let bookmark = Bookmark::default();
        let tests = vec![("1 + 2", Value::Number(3.))];
        run_tests(tests, &bookmark)
    }
}
