use super::Value;
use crate::{Bookmark, Error, Result};
use pest::{
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
    Parser,
};

lazy_static! {
    /// Static climber to be reused each `eval` call.
    /// Defines order of operations (PEMDAS, then comparator, then conjunctions).
    static ref PARSER: PrattParser<Rule> = PrattParser::new()
    .op(Op::infix(Rule::And, Assoc::Left) | Op::infix(Rule::Or, Assoc::Left))
    .op(Op::infix(Rule::Eq, Assoc::Left)
        | Op::infix(Rule::Neq, Assoc::Left)
        | Op::infix(Rule::Lt, Assoc::Left)
        | Op::infix(Rule::Leq, Assoc::Left)
        | Op::infix(Rule::Gt, Assoc::Left)
        | Op::infix(Rule::Geq, Assoc::Left))
    .op(Op::infix(Rule::Add, Assoc::Left) | Op::infix(Rule::Sub, Assoc::Left))
    .op(Op::infix(Rule::Mul, Assoc::Left) | Op::infix(Rule::Div, Assoc::Left));
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
        // Define lambdas for use by the parser.
        let primary = |pair| Self::eval_expr(pair, bookmark);
        let infix = |lhs: Result<Value>, op: Pair<Rule>, rhs: Result<Value>| match (lhs, rhs) {
            (Ok(lhs), Ok(rhs)) => Self::eval_binary_expr(lhs, op, rhs),
            (Ok(_), Err(rhs_err)) => Err(rhs_err),
            (Err(lhs_err), _) => Err(lhs_err),
        };

        match pair.as_rule() {
            Rule::BinaryExpression => PARSER
                .map_primary(primary)
                .map_infix(infix)
                .parse(pair.into_inner()),
            Rule::UnaryExpression => {
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
            Rule::Bool | Rule::Number | Rule::QuotedString | Rule::UnquotedString => {
                Value::from_yml(pair.as_str())
            }
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
        Ok(result)
    }

    /// Evaluates a unary expression.
    fn eval_unary_expr(op: Pair<Rule>, value: Value) -> Result<Value> {
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
        let bookmark = Bookmark::new(hashmap! {
            "test".to_string() => hashmap! {
                "var1".to_string() => Value::Number(1.0)
            },
            "global".to_string() => hashmap! {
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
            ("not true and true", Value::Bool(false)),
            ("true and not false", Value::Bool(true)),
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
            ("1.5 + 2.5", Value::Number(4.0)),
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
