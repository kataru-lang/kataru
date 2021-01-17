use kataru_parser::State;
use regex::{Captures, Regex};
use std::borrow::Cow;

pub fn replace_vars(text: &str, vars: &State) -> String {
    lazy_static! {
        static ref VARS_RE: Regex = Regex::new(r"\$\{([a-zA-Z0-9_]*)\}").unwrap();
    }
    VARS_RE
        .replace_all(&text, |cap: &Captures| {
            let var = &cap[1];
            match vars.get(var) {
                Some(value) => Cow::from(value.to_string()),
                None => Cow::from(format!("${{{}}}", var).to_string()),
            }
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kataru_parser::Value;

    #[test]
    fn test_str_replace() {
        let mut state = State::new();
        state.insert("var1".to_string(), Value::Number(1.0));
        state.insert("var2".to_string(), Value::String("a".to_string()));

        assert_eq!(
            replace_vars("var1 = ${var1} and var2 = ${var2}. This costs $10.", &state),
            "var1 = 1 and var2 = a. This costs $10."
        )
    }

    #[test]
    fn test_invalid_vars() {
        let state = State::new();
        assert_eq!(replace_vars("var1 = ${var1}.", &state), "var1 = ${var1}.")
    }
}
