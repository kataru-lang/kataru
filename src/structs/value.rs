use crate::{
    error::{Error, Result},
    Bookmark, SINGLE_VAR_RE, VARS_RE,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{AddAssign, SubAssign};

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::Bool(v) => write!(f, "{}", v),
        }
    }
}

impl AddAssign<&Self> for Value {
    fn add_assign(&mut self, rhs: &Self) {
        match (&self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => *self = Self::Number(n1 + n2),
            _ => (),
        }
    }
}

impl SubAssign<&Self> for Value {
    fn sub_assign(&mut self, rhs: &Self) {
        match (&self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => *self = Self::Number(*n1 - n2),
            _ => *self = Self::Number(0 as f64),
        }
    }
}

impl Value {
    pub fn same_type(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Value::Bool(_), Value::Bool(_)) => true,
            (Value::Number(_), Value::Number(_)) => true,
            (Value::String(_), Value::String(_)) => true,
            _ => false,
        }
    }

    pub fn from_yml(yaml_value: serde_yaml::Value) -> Result<Self> {
        match yaml_value {
            serde_yaml::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_yaml::Value::String(s) => Ok(Value::String(s)),
            serde_yaml::Value::Number(n) => Ok(Value::Number(n.as_f64().unwrap())),
            _ => Err(error!("Cannot create value from {:?}", yaml_value)),
        }
    }

    /// Parses a single piece of text into a value;
    pub fn parse(text: &str) -> Result<Self> {
        match serde_yaml::from_str(&text) {
            Ok(r) => Self::from_yml(r),
            Err(e) => Err(error!("{}", e)),
        }
    }

    fn eval_bool_var(var: &str, bookmark: &Bookmark) -> Result<Option<bool>> {
        for cap in SINGLE_VAR_RE.captures_iter(var) {
            if let Value::Bool(bool) = bookmark.value(&cap[1])? {
                return Ok(Some(*bool));
            } else {
                return Err(error!("Invalid boolean variable '${}'", var));
            }
        }
        Ok(None)
    }

    fn get_compared_val(var: &str, bookmark: &Bookmark) -> Result<Value> {
        for cap in SINGLE_VAR_RE.captures_iter(var) {
            return Ok(bookmark.value(&cap[1])?.clone());
        }
        Value::parse(var)
    }

    fn eval_comparator(var1: &str, var2: &str, cmp: &str, bookmark: &Bookmark) -> Result<bool> {
        let val1 = Self::get_compared_val(var1, bookmark)?;
        let val2 = Self::get_compared_val(var2, bookmark)?;
        match cmp {
            "==" => Ok(val1 == val2),
            "!=" => Ok(val1 != val2),
            "<" => Ok(val1 < val2),
            "<=" => Ok(val1 <= val2),
            ">" => Ok(val1 > val2),
            ">=" => Ok(val1 >= val2),
            _ => Err(error!("Invalid comparator '{}'", cmp)),
        }
    }

    /// A bool expr can be of the form `$var`, `not $var`, or `$var cmp X`.
    fn eval_bool_expr(expr: &str, bookmark: &Bookmark) -> Result<bool> {
        // If singular $var expression
        if let Some(bool) = Self::eval_bool_var(expr, bookmark)? {
            return Ok(bool);
        }

        // If $var CMP $var / $var CMP val
        let split: Vec<&str> = expr.split(' ').collect();
        match split.as_slice() {
            ["not", var] => {
                if let Some(bool) = Self::eval_bool_var(var, bookmark)? {
                    Ok(!bool)
                } else {
                    Err(error!("Invalid boolean expr after 'not'."))
                }
            }
            [var1, cmp, var2] => Self::eval_comparator(var1, var2, cmp, bookmark),
            _ => Err(error!("Invalid boolean expr '{}'", expr)),
        }
    }

    fn eval_or_exprs(exprs: &Vec<&str>, bookmark: &Bookmark) -> Result<bool> {
        let mut bool = false;
        for expr in exprs {
            bool |= Self::eval_bool_expr(expr, bookmark)?;
        }
        Ok(bool)
    }

    fn eval_and_exprs(exprs: &Vec<&str>, bookmark: &Bookmark) -> Result<bool> {
        let mut bool = true;
        for expr in exprs {
            bool &= Self::eval_bool_expr(expr, bookmark)?;
        }
        Ok(bool)
    }

    /// Evaluates a string that may contain $variable expressions.
    /// If invalid expression, returns Ok(None).
    /// If the expression is valid, but contains invalid references, returns Err(...).
    pub fn eval_bool_exprs(expr: &str, bookmark: &Bookmark) -> Result<bool> {
        // First get highest level, boolean operators.
        // Try multiple "and" clauses.
        {
            let exprs: Vec<&str> = expr.split(" and ").collect();
            if exprs.len() > 1 {
                return Self::eval_and_exprs(&exprs, bookmark);
            }
        }

        // Next try "or" clauses
        {
            let exprs: Vec<&str> = expr.split(" or ").collect();
            if exprs.len() > 1 {
                return Self::eval_or_exprs(&exprs, bookmark);
            }
        }

        // Default to try to parse the whole thing as a bool expression.
        Self::eval_bool_expr(expr, bookmark)
    }

    /// Evaluates a string that may contain $variable expressions.
    /// If invalid expression, returns Ok(None).
    /// If the expression is valid, but contains invalid references, returns Err(...).
    pub fn eval(expr: &str, bookmark: &Bookmark) -> Result<Option<Value>> {
        // If just a single variable, return the value.
        for cap in SINGLE_VAR_RE.captures_iter(expr) {
            let value = bookmark.value(&cap[1])?.clone();
            return Ok(Some(value));
        }

        // Otherwise try evaluating a boolean expression.
        let value = Value::Bool(Self::eval_bool_exprs(expr, bookmark)?);
        Ok(Some(value))
    }

    /// Attempts to evaluate a value as an expression.
    /// For all types except string, this is a no-op.
    /// If a string type is not a valid expression, does nothing.
    pub fn eval_in_place(&mut self, bookmark: &Bookmark) -> Result<()> {
        if let Value::String(expr) = self {
            // Only if the string can be treated as a valid expression do we replace.
            if !VARS_RE.is_match(expr) {
                return Ok(());
            }
            if let Some(new_val) = Self::eval(expr, bookmark)? {
                *self = new_val;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bookmark, Position};

    #[test]
    fn test_eval() {
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

        {
            let val = Value::eval("$var1", &bookmark).unwrap().unwrap();
            assert_eq!(val, Value::Number(1.));
        }

        {
            let val = Value::eval("$b0 and $b1", &bookmark).unwrap().unwrap();
            assert_eq!(val, Value::Bool(false));
        }

        {
            let val = Value::eval("not $b0 and $b1", &bookmark).unwrap().unwrap();
            assert_eq!(val, Value::Bool(true));
        }

        {
            let val = Value::eval("$b0 or $b1", &bookmark).unwrap().unwrap();
            assert_eq!(val, Value::Bool(true));
        }

        {
            let val = Value::eval("$b0 or not $b1", &bookmark).unwrap().unwrap();
            assert_eq!(val, Value::Bool(false));
        }
    }
}
