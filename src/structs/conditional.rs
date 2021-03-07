use super::{Bookmark, Comparator, Value};
use crate::error::{Error, Result};
use crate::traits::FromStr;

#[derive(Debug, PartialEq)]
pub struct Conditional<'a> {
    pub var: &'a str,
    pub cmp: Comparator,
    pub val: Value,
}

impl<'a> Conditional<'a> {
    pub fn eval(&self, bookmark: &Bookmark) -> Result<bool> {
        self.cmp(bookmark.value(&self.var)?)
    }

    pub fn cmp(&self, val: &Value) -> Result<bool> {
        if !val.same_type(&self.val) {
            return Err(error!(
                "Comparisons require values of the same type, not {:?} and {:?}",
                val, self.val
            ));
        }
        match self.cmp {
            Comparator::EQ => Ok(val == &self.val),
            Comparator::NEQ => Ok(val != &self.val),
            Comparator::LT => Ok(val < &self.val),
            Comparator::LEQ => Ok(val <= &self.val),
            Comparator::GT => Ok(val > &self.val),
            Comparator::GEQ => Ok(val <= &self.val),
        }
    }
}

impl<'a> FromStr<'a> for Conditional<'a> {
    fn from_str(text: &'a str) -> Result<Self> {
        let split: Vec<&'a str> = text.split(' ').collect();
        if split[0] == "if" || split[0] == "elif" {
            if split.len() == 4 {
                return Ok(Self {
                    var: &split[1][1..],
                    cmp: Comparator::from_str(split[2])?,
                    val: Value::parse(split[3])?,
                });
            } else if split.len() == 2 {
                return Ok(Self {
                    var: &split[1][1..],
                    cmp: Comparator::EQ,
                    val: Value::Bool(true),
                });
            } else if split.len() == 3 && split[1] == "not" {
                return Ok(Self {
                    var: &split[2][1..],
                    cmp: Comparator::EQ,
                    val: Value::Bool(false),
                });
            }
        }
        Err(error!(
            "Conditionals must be of the form 'if $VAR [<,<=,>,=>,==,] VALUE:' or 'if [not] $VAR', not '{}'",
            text
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_cmp() {
        let v1 = Value::Number(1.);
        let v2 = Value::Number(2.);
        assert_eq!(v1 < v2, true);

        let v1 = Value::Number(1.);
        let v2 = Value::String("test".to_string());
        assert_eq!(v1 < v2, false);
    }

    /// Tests construction and comparison of conditional
    #[test]
    fn test_cond_cmp() {
        let res = Conditional::from_str("if $var > 5");
        assert!(res.is_ok(), "Parsing failed: {:?}", res.unwrap_err());

        let cond = res.unwrap();
        assert_eq!(
            cond,
            Conditional {
                var: "var",
                val: Value::Number(5.0),
                cmp: Comparator::GT
            }
        );
    }
}
