mod operators;
mod parser;

use crate::{
    error::{Error, Result},
    Bookmark,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
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

    pub fn from_yml_value(yaml_value: serde_yaml::Value) -> Result<Self> {
        match yaml_value {
            serde_yaml::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_yaml::Value::String(s) => Ok(Value::String(s)),
            serde_yaml::Value::Number(n) => Ok(Value::Number(n.as_f64().unwrap())),
            _ => Err(error!("Cannot create value from {:?}", yaml_value)),
        }
    }

    /// Parses a single piece of text into a value;
    pub fn from_yml(text: &str) -> Result<Self> {
        match serde_yaml::from_str(&text) {
            Ok(r) => Self::from_yml_value(r),
            Err(e) => Err(error!("{}", e)),
        }
    }

    /// Gets a value from a variable. Assumes that the $ has already be stripped.
    pub fn from_var(var: &str, bookmark: &Bookmark) -> Result<Self> {
        Ok(bookmark.value(var)?.clone())
    }

    /// Gets truthy value.
    pub fn to_bool(self) -> Result<bool> {
        match self {
            Self::Bool(b) => Ok(b),
            _ => Err(error!("'{}' was not a boolean expression.", self)),
        }
    }

    fn extract_conditional_expr(expr: &str) -> &str {
        static IF_PREFIX: &str = "if ";
        static ELIF_PREFIX: &str = "elif ";
        if expr.starts_with(IF_PREFIX) {
            &expr[IF_PREFIX.len()..]
        } else if expr.starts_with(ELIF_PREFIX) {
            &expr[ELIF_PREFIX.len()..]
        } else {
            ""
        }
    }

    pub fn from_conditional(expr: &str, bookmark: &Bookmark) -> Result<bool> {
        Self::from_expr(Self::extract_conditional_expr(expr), bookmark)?.to_bool()
    }
}
