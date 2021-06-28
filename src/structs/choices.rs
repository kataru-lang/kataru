use super::{line_len, Bookmark, RawLine};
use crate::{error::Result, Map, Value};
use linear_map::LinearMap;
use serde::{Deserialize, Serialize};

const EMPTY_STRING: &String = &String::new();

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ChoiceTarget {
    Lines(Vec<RawLine>),
    PassageName(String),
    None,
}
impl Default for ChoiceTarget {
    fn default() -> Self {
        Self::None
    }
}
impl ChoiceTarget {
    pub fn line_len(&self) -> usize {
        match self {
            Self::Lines(lines) => line_len(lines),
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawChoice {
    Conditional(LinearMap<String, ChoiceTarget>),
    Target(ChoiceTarget),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RawChoices {
    choices: LinearMap<String, RawChoice>,
    #[serde(default)]
    pub timeout: f64,
    #[serde(default)]
    pub default: ChoiceTarget,
}
impl RawChoices {
    /// Returns number of choices.
    pub fn len(&self) -> usize {
        self.choices.len()
    }

    /// Returns equivalent number of lines for the embedded passages.
    /// If this choices object has no embedded passages, `line_len(choices) == 1`.
    /// Otherwise it's `1 + the line length of each embedded passage + number of embedded passages`.
    /// This includes all conditionals.
    pub fn line_len(&self) -> usize {
        let mut length = 1 + self.default.line_len();
        for (_key, choice) in &self.choices {
            match choice {
                RawChoice::Target(ChoiceTarget::Lines(lines)) => {
                    length += line_len(lines) + 1;
                }
                RawChoice::Conditional(conditional) => {
                    for (_inner_key, target) in conditional {
                        if let ChoiceTarget::Lines(lines) = target {
                            length += line_len(lines) + 1;
                        }
                    }
                }
                _ => (),
            }
        }
        length
    }
    pub fn take(&self, bookmark: &mut Bookmark, skip_lines: usize) -> usize {
        let next_line = bookmark.line() + self.line_len() - skip_lines;
        bookmark.skip_lines(skip_lines);
        next_line
    }
}
impl<'a> IntoIterator for &'a RawChoices {
    type Item = (&'a String, &'a RawChoice);
    type IntoIter = linear_map::Iter<'a, String, RawChoice>;

    fn into_iter(self) -> Self::IntoIter {
        self.choices.iter()
    }
}

/// Public interface to choices, just gives choice names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Choices {
    // List of choices matching the order of the raw chocies.
    pub choices: Vec<String>,
    #[serde(default)]
    pub timeout: f64,
}
impl Choices {
    pub fn push(&mut self, choice: &str) {
        self.choices.push(choice.to_string());
    }
    pub fn clear(&mut self) {
        self.choices.clear()
    }
    pub fn reserve(&mut self, additional: usize) {
        self.choices.reserve(additional)
    }
    pub fn reverse(&mut self) {
        self.choices.reverse()
    }
    pub fn len(&self) -> usize {
        self.choices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.choices.is_empty()
    }

    /// Repopulates the `choice_to_passage` map with all valid choices.
    pub fn from_raw<'r>(
        choice_to_passage: &mut Map<&'r str, &'r str>,
        choice_to_line_num: &mut Map<&'r str, usize>,
        raw: &'r RawChoices,
        bookmark: &Bookmark,
    ) -> Result<Self> {
        let mut choices = Self::default();
        choices.timeout = raw.timeout;
        choices.reserve(raw.len());

        // Reset structs.
        choice_to_passage.clear();
        choice_to_passage.reserve(raw.len());

        //  The current passage target.
        let mut passage: &String = &EMPTY_STRING;
        let mut line_num = raw.line_len() - raw.default.line_len();
        let mut add_target = |key: &'r str, target: &'r ChoiceTarget| {
            match target {
                // Populate unconditional level choices.
                ChoiceTarget::PassageName(passage_name) => {
                    passage = passage_name;
                    choices.push(key);
                    choice_to_passage.insert(key, passage);
                }
                // Infer which passage this refers to.
                ChoiceTarget::None => {
                    choices.push(key);
                    choice_to_passage.insert(key, passage);
                }
                ChoiceTarget::Lines(lines) => {
                    choices.push(key);
                    line_num -= line_len(lines) + 1;
                    choice_to_line_num.insert(key, line_num);
                }
            }
        };
        // Populate through valid choices and infer implicit passage targets.
        for (key, choice) in raw.into_iter().rev() {
            match choice {
                RawChoice::Target(target) => add_target(key, target),
                // Populate all choices are behind a true conditional.
                RawChoice::Conditional(conditional) => {
                    if !Value::from_conditional(key, bookmark)? {
                        continue;
                    }
                    for (inner_key, target) in conditional.iter().rev() {
                        add_target(inner_key, target);
                    }
                }
            }
        }

        // Since we iterated backwards for populating chocies, we must reverse to match order.
        choices.reverse();
        Ok(choices)
    }
}
impl<'a> IntoIterator for &'a Choices {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.choices.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FromYaml;

    impl FromYaml for RawChoices {}

    #[test]
    fn test_choices_order() {
        let bookmark = Bookmark::new(hashmap! {});
        let choices_str = r#"
            choices:
              a: A
              b: B
              c:
              d: D
              e:
                - E1
              f:
                - F1
                - F2
            default:
              - default1
              - default2
        "#;

        let raw = RawChoices::from_yml(choices_str).unwrap();
        let mut choice_to_passage = Map::default();
        let mut choice_to_line_num = Map::default();

        let choices = Choices::from_raw(
            &mut choice_to_passage,
            &mut choice_to_line_num,
            &raw,
            &bookmark,
        )
        .unwrap();
        assert_eq!(
            choices.choices,
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
                "f".to_string()
            ]
        );
        assert_eq!(
            choice_to_passage,
            hashmap! {
                "a" => "A",
                "b" => "B",
                "c" => "D",
                "d" => "D",
            }
        );

        assert_eq!(
            choice_to_line_num,
            hashmap! {
                "e" => 1,
                "f" => 3,
            }
        );
    }
}
