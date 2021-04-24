use crate::{Bookmark, Error, Result, Value};
use pest::{
    iterators::Pair,
    prec_climber::{Assoc, Operator as PrecOp, PrecClimber},
    Parser,
};

lazy_static! {
    /// Static climber to be reused each `eval` call.
    /// Defines order of operations (PEMDAS, then comparator, then conjunctions).
    static ref CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        PrecOp::new(Rule::Add, Assoc::Left) | PrecOp::new(Rule::Sub, Assoc::Left),
        PrecOp::new(Rule::Mul, Assoc::Left) | PrecOp::new(Rule::Div, Assoc::Left),
        PrecOp::new(Rule::Eq, Assoc::Left)
            | PrecOp::new(Rule::Neq, Assoc::Left)
            | PrecOp::new(Rule::Lt, Assoc::Left)
            | PrecOp::new(Rule::Leq, Assoc::Left)
            | PrecOp::new(Rule::Gt, Assoc::Left)
            | PrecOp::new(Rule::Geq, Assoc::Left),
        PrecOp::new(Rule::And, Assoc::Left) | PrecOp::new(Rule::Or, Assoc::Left),
    ]);
}
/// Pest parser generated from ast/grammar.pest.
#[derive(pest_derive::Parser)]
#[grammar = "ast/grammar.pest"]
struct ExprParser;

/// Evaluates an expression `expr`. Uses `bookmark` for $variable lookup.
pub fn eval(expr: &str, bookmark: &Bookmark) -> Result<Value> {
    let pair = ExprParser::parse(Rule::Expr, expr).unwrap().next().unwrap();
    eval_expr(pair, bookmark)
}

/// Evaluates an expression from a `Pair` tree.
fn eval_expr<'i>(pair: Pair<'i, Rule>, bookmark: &Bookmark) -> Result<Value> {
    // Define lambdas for use by precedence climber.
    let primary = |pair| eval_expr(pair, bookmark);
    let infix = |lhs: Result<Value>, op: Pair<Rule>, rhs: Result<Value>| {
        if let (Ok(lhs), Ok(rhs)) = (lhs, rhs) {
            eval_binary_expr(lhs, op, rhs)
        } else {
            Err(error!("No global namespace"))
        }
    };

    match pair.as_rule() {
        Rule::BinaryExpr => CLIMBER.climb(pair.into_inner(), primary, infix),
        Rule::UnaryExpr => {
            let mut it = pair.into_inner();
            let op = it.next();
            let inner = it.next();

            if let (Some(op_pair), Some(inner_pair)) = (op, inner) {
                let value = eval_expr(inner_pair, bookmark)?;
                eval_unary_expr(op_pair, value)
            } else {
                Err(error!("Invalid Unary"))
            }
        }
        Rule::Variable => Value::from_var(pair.as_str(), bookmark),
        Rule::Value | Rule::String => Value::from_str(pair.as_str()),
        _ => {
            return Ok(Value::Number(0.));
        }
    }
}

/// Evaluates a binary expression.
fn eval_binary_expr(lhs: Value, op: Pair<Rule>, rhs: Value) -> Result<Value> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bookmark, Position, Value};

    #[test]
    fn parse_expr() {
        let bookmark = Bookmark {
            position: Position {
                namespace: "test".to_string(),
                passage: "".to_string(),
                line: 0,
            },
            state: btreemap! {
                "test".to_string() => btreemap! {
                    "var1".to_string() => Value::Number(1.0)
                },
                "global".to_string() => btreemap! {
                    "b0".to_string() => Value::Bool(false),
                    "b1".to_string() => Value::Bool(true),
                    "var2".to_string() => Value::String("a".to_string()),
                    "char.var1".to_string() => Value::String("b".to_string())
                }
            },
            stack: Vec::new(),
            snapshots: btreemap! {},
        };

        let tests = vec![
            ("value", Value::String("value".to_string())),
            ("1 + 2", Value::Number(3.)),
            ("2 * 1 + 4", Value::Number(6.)),
            ("2 * (1 + 4)", Value::Number(10.)),
            ("- (1 / 3)", Value::Number(-1. / 3.)),
            ("true and false", Value::Bool(false)),
            ("true or false", Value::Bool(true)),
            ("1 < 2", Value::Bool(true)),
            ("$var2 == a", Value::Bool(true)),
            ("$var2 != a", Value::Bool(false)),
            ("$char.var1 == b", Value::Bool(true)),
            ("a + b", Value::String("ab".to_string())),
            ("not true", Value::Bool(false)),
        ];

        for (expr, expected) in tests {
            assert_eq!(expected, eval(expr, &bookmark).unwrap());
        }
    }
}
