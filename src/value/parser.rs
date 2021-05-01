use super::Value;
use crate::{Bookmark, Error, Result};
use pest::{
    iterators::Pair,
    prec_climber::{Assoc, Operator as PrecOp, PrecClimber},
    Parser,
};

lazy_static! {
    /// Static climber to be reused each `eval` call.
    /// Defines order of operations (PEMDAS, then comparator, then conjunctions).
    static ref CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        PrecOp::new(Rule::And, Assoc::Left) | PrecOp::new(Rule::Or, Assoc::Left),
        PrecOp::new(Rule::Eq, Assoc::Left)
            | PrecOp::new(Rule::Neq, Assoc::Left)
            | PrecOp::new(Rule::Lt, Assoc::Left)
            | PrecOp::new(Rule::Leq, Assoc::Left)
            | PrecOp::new(Rule::Gt, Assoc::Left)
            | PrecOp::new(Rule::Geq, Assoc::Left),
        PrecOp::new(Rule::Add, Assoc::Left) | PrecOp::new(Rule::Sub, Assoc::Left),
        PrecOp::new(Rule::Mul, Assoc::Left) | PrecOp::new(Rule::Div, Assoc::Left),
    ]);
}

/// Pest parser generated from ast/grammar.pest.
#[derive(pest_derive::Parser)]
#[grammar = "value/grammar.pest"]
struct ExprParser;

impl Value {
    /// Evaluates an expression `expr`. Uses `bookmark` for $variable lookup.
    pub fn from_expr(expr: &str, bookmark: &Bookmark) -> Result<Self> {
        let mut pairs = ExprParser::parse(Rule::Program, expr)?;
        if let Some(pair) = pairs.next() {
            Self::eval_expr(pair, bookmark)
        } else {
            Err(Error::Pest("Invalid expression.".to_string()))
        }
    }

    /// Evaluates an expression from a `Pair` tree.
    fn eval_expr<'i>(pair: Pair<'i, Rule>, bookmark: &Bookmark) -> Result<Value> {
        // Define lambdas for use by precedence climber.
        let primary = |pair| Self::eval_expr(pair, bookmark);
        let infix = |lhs: Result<Value>, op: Pair<Rule>, rhs: Result<Value>| match (lhs, rhs) {
            (Ok(lhs), Ok(rhs)) => Self::eval_binary_expr(lhs, op, rhs),
            (Ok(_), Err(rhs_err)) => Err(rhs_err),
            (Err(lhs_err), _) => Err(lhs_err),
        };

        match pair.as_rule() {
            Rule::BinaryExpr => CLIMBER.climb(pair.into_inner(), primary, infix),
            Rule::UnaryExpr => {
                let mut it = pair.into_inner();
                let op = it.next();
                let inner = it.next();

                if let (Some(op_pair), Some(inner_pair)) = (op, inner) {
                    let value = Self::eval_expr(inner_pair, bookmark)?;
                    Self::eval_unary_expr(op_pair, value)
                } else {
                    Err(error!("Invalid Unary"))
                }
            }
            Rule::Variable => Value::from_var(pair.as_str(), bookmark),
            Rule::Value | Rule::String => Value::from_yml(pair.as_str()),
            _ => {
                return Ok(Value::Number(0.));
            }
        }
    }

    /// Evaluates a binary expression.
    fn eval_binary_expr(lhs: Value, op: Pair<Rule>, rhs: Value) -> Result<Value> {
        // print!("binary expr: ({} {} {}) ", lhs, op.as_str(), rhs);
        let result = match op.as_rule() {
            Rule::Add => lhs + rhs,
            Rule::Sub => lhs - rhs,
            Rule::Mul => lhs * rhs,
            Rule::Div => lhs / rhs,
            Rule::And => lhs & rhs,
            Rule::Or => lhs | rhs,
            Rule::Lt => Value::Bool(lhs < rhs),
            Rule::Leq => Value::Bool(lhs <= rhs),
            Rule::Gt => Value::Bool(lhs > rhs),
            Rule::Geq => Value::Bool(lhs >= rhs),
            Rule::Eq => Value::Bool(lhs == rhs),
            Rule::Neq => Value::Bool(lhs != rhs),
            _ => {
                return Err(error!("Invalid binary expression."));
            }
        };
        // println!(" = {}", result);
        Ok(result)
    }

    /// Evaluates a unary expression.
    fn eval_unary_expr(op: Pair<Rule>, value: Value) -> Result<Value> {
        // println!("unary expr: {} {}", op, value);
        let result = match op.as_rule() {
            Rule::Not => !value,
            Rule::Add => value,
            Rule::Sub => -value,
            _ => return Err(error!("Invalid unary expression.")),
        };
        Ok(result)
    }

    pub fn eval_as_expr(&mut self, bookmark: &Bookmark) -> Result<()> {
        let result = match self {
            Self::String(expr) => Self::from_expr(expr, bookmark),
            _ => return Ok(()),
        };
        match result {
            Ok(value) => *self = value,
            Err(Error::Pest(_)) => (), // Pest errors mean this was a normal string, not an expr.
            Err(e) => return Err(e),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bookmark, Error, Value};

    #[test]
    fn test_parse_expr() {
        let bookmark = Bookmark::new(btreemap! {
            "test".to_string() => btreemap! {
                "var1".to_string() => Value::Number(1.0)
            },
            "global".to_string() => btreemap! {
                "b0".to_string() => Value::Bool(false),
                "b1".to_string() => Value::Bool(true),
                "var2".to_string() => Value::String("a".to_string()),
                "char.var1".to_string() => Value::String("b".to_string())
            }
        });

        let tests = vec![
            ("value", Value::String("value".to_string())),
            ("1 + 2", Value::Number(3.)),
            ("2 * 1 + 4", Value::Number(6.)),
            ("2 * (1 + 4)", Value::Number(10.)),
            ("- (1 / 3)", Value::Number(-1. / 3.)),
            ("true and false", Value::Bool(false)),
            ("true or false", Value::Bool(true)),
            ("1 < 2", Value::Bool(true)),
            (
                "$test:var1 + 1 > 0 and $test:var1 + 1 < 3",
                Value::Bool(true),
            ),
            ("$var2 == a", Value::Bool(true)),
            ("$var2 != a", Value::Bool(false)),
            ("$char.var1 == b", Value::Bool(true)),
            ("a + b", Value::String("ab".to_string())),
            ("not true", Value::Bool(false)),
        ];

        for (expr, expected) in tests {
            assert_eq!(expected, Value::from_expr(expr, &bookmark).unwrap());
        }
    }

    #[test]
    fn test_invalid_expr() {
        let bookmark = Bookmark::default();
        let expr = "this is a string";
        let result = Value::from_expr(expr, &bookmark);
        assert!(matches!(result, Err(Error::Pest(_))));
    }
}
