use crate::{
    contains_var,
    error::{Error, Result},
    extract_var, Bookmark,
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

    fn eval_bool_var(text: &str, bookmark: &Bookmark) -> Result<Option<bool>> {
        if let Some(var) = extract_var(text) {
            if let Value::Bool(bool) = Value::from_var(var, bookmark)? {
                return Ok(Some(bool));
            } else {
                return Err(error!("Invalid boolean variable '${}'", var));
            }
        }
        Ok(None)
    }

    /// Gets a value from a variable. Assumes that the $ has already be stripped.
    fn from_var(var: &str, bookmark: &Bookmark) -> Result<Self> {
        Ok(bookmark.value(var)?.clone())
    }

    /// If `token` is a variable, returns that variable's value.
    /// Otherwise parses `token` as a yaml literal.
    /// Raises an error if unable to parse or if the varname doesn't exist.
    fn from_token(token: &str, bookmark: &Bookmark) -> Result<Self> {
        if let Some(var) = extract_var(token) {
            return Self::from_var(var, bookmark);
        }
        Value::parse(token)
    }

    fn eval_comparator(token1: &str, token2: &str, cmp: &str, bookmark: &Bookmark) -> Result<bool> {
        let val1 = Self::from_token(token1, bookmark)?;
        let val2 = Self::from_token(token2, bookmark)?;
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
        // Handle negation
        let mut negate = false;
        let not_prefix = "not ";
        let expr = if expr.starts_with(not_prefix) {
            negate = true;
            &expr[not_prefix.len()..]
        } else {
            expr
        };

        // If singular $var expression
        if let Some(bool) = Self::eval_bool_var(expr, bookmark)? {
            return Ok(negate ^ bool);
        }

        // If $var CMP $var / $var CMP val
        let split: Vec<&str> = expr.split(' ').collect();
        match split.as_slice() {
            [var1, cmp, var2] => Ok(negate ^ Self::eval_comparator(var1, var2, cmp, bookmark)?),
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
    pub fn eval(expr: &str, bookmark: &Bookmark) -> Result<Self> {
        // If just a single variable, return the value.
        if let Some(var) = extract_var(expr) {
            return Ok(Self::from_var(var, bookmark)?);
        }

        // Otherwise try evaluating a boolean expression.
        let value = Value::Bool(Self::eval_bool_exprs(expr, bookmark)?);
        Ok(value)
    }

    /// If this value is actually an expression that needs to be evaluated,
    /// return Some(&str) containing the expression.
    /// Otherwise return None.
    pub fn get_expr(&self) -> Option<&str> {
        if let Value::String(expr) = self {
            if contains_var(expr) {
                Some(&expr)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// If this value is a varaible, gets that variable's name.
    /// Otherwise None.
    pub fn get_var(&self) -> Option<&str> {
        if let Value::String(text) = self {
            if let Some(var) = extract_var(text) {
                Some(&var)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Attempts to evaluate a value as an expression.
    /// For all types except string, this is a no-op.
    /// If a string type is not a valid expression, does nothing.
    pub fn eval_in_place(&mut self, bookmark: &Bookmark) -> Result<()> {
        if let Some(expr) = self.get_expr() {
            *self = Self::eval(expr, bookmark)?
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
            let val = Value::eval("$var1", &bookmark).unwrap();
            assert_eq!(val, Value::Number(1.));
        }

        {
            let val = Value::eval("$b0 and $b1", &bookmark).unwrap();
            assert_eq!(val, Value::Bool(false));
        }

        {
            let val = Value::eval("not $b0 and $b1", &bookmark).unwrap();
            assert_eq!(val, Value::Bool(true));
        }

        {
            let val = Value::eval("$b0 or $b1", &bookmark).unwrap();
            assert_eq!(val, Value::Bool(true));
        }

        {
            let val = Value::eval("$b0 or not $b1", &bookmark).unwrap();
            assert_eq!(val, Value::Bool(false));
        }
    }
}
