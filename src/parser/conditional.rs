use super::{Comparator, Parsable, State, Value};
use crate::ValidationError;

#[derive(Debug, PartialEq)]
pub struct Conditional<'a> {
    pub var: &'a str,
    pub cmp: Comparator,
    pub val: Value,
}

impl<'a> Parsable<'a> for Conditional<'a> {
    fn parse(text: &'a str) -> Result<Self, ValidationError> {
        let split: Vec<&'a str> = text.split(' ').collect();
        if split.len() != 4 || split[0] != "if" {
            return Err(verror!(
                "Conditionals must be of the form 'if VAR [<,<=,>,=>,==,] VALUE:', not {}",
                text
            ));
        }
        Ok(Self {
            var: split[1],
            cmp: Comparator::parse(split[2])?,
            val: Value::parse(split[3])?,
        })
    }
}

impl<'a> Conditional<'a> {
    pub fn eval(&self, state: &State) -> Result<bool, ValidationError> {
        self.cmp(&state[self.var])
    }

    pub fn cmp(&self, val: &Value) -> Result<bool, ValidationError> {
        if !val.same_type(&self.val) {
            return Err(verror!(
                "Comparisons require values of the same type, not {:?} and {:?}",
                val,
                self.val
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
        let res = Conditional::parse("if var > 5");
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
