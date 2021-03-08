use crate::structs::Bookmark;
use crate::{error::Result, Value};
use regex::{Captures, Regex};
use std::borrow::Cow;

lazy_static! {
    static ref VARS_RE: Regex = Regex::new(r"\$([A-Za-z]+[:\w\.]*)\b").unwrap();
    static ref BRACKET_VARS_RE: Regex = Regex::new(r"\{\$([A-Za-z]+[:\w\.]*)\}").unwrap();
}

/// This is a line with var=${var} and var2=${var2}
pub fn replace_vars(text: &str, bookmark: &Bookmark) -> String {
    let vars_replaced = BRACKET_VARS_RE.replace_all(&text, |cap: &Captures| {
        let var = &cap[1];
        match bookmark.value(var) {
            Ok(value) => Cow::from(value.to_string()),
            Err(_) => Cow::from(format!("{{${}}}", var).to_string()),
        }
    });

    VARS_RE
        .replace_all(&vars_replaced, |cap: &Captures| {
            let var = &cap[1];
            match bookmark.value(var) {
                Ok(value) => Cow::from(value.to_string()),
                Err(_) => Cow::from(format!("${}", var).to_string()),
            }
        })
        .to_string()
}

pub fn replace_var(text: &str, bookmark: &Bookmark) -> Result<Option<Value>> {
    if text.starts_with("$") {
        Ok(Some(bookmark.value(&text[1..text.len()])?.clone()))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Position, Value};

    #[test]
    fn test_str_replace() {
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
                    "var2".to_string() => Value::String("a".to_string()),
                    "char.var1".to_string() => Value::String("b".to_string())
                }
            },
            stack: Vec::new(),
            snapshots: btreemap! {},
        };
        assert_eq!(
            replace_vars(
                "var1 = {$var1}, var2 = {$global:var2}, char.var1 = $char.var1. Tickets cost $10.",
                &bookmark
            ),
            "var1 = 1, var2 = a, char.var1 = b. Tickets cost $10."
        )
    }

    #[test]
    fn test_invalid_vars() {
        let bookmark = Bookmark::default();
        assert_eq!(
            replace_vars("var1 = ${var1}.", &bookmark),
            "var1 = ${var1}."
        )
    }
}
