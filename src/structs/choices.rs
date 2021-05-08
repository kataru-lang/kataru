use std::iter::Rev;

use super::Bookmark;
use crate::{error::Result, traits::MoveValues, Value};
use linear_map::LinearMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawChoice {
    Conditional(LinearMap<String, Option<String>>),
    PassageName(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RawChoices {
    choices: LinearMap<String, RawChoice>,
    #[serde(default)]
    pub timeout: f64,
}

impl<'a> IntoIterator for &'a RawChoices {
    type Item = (&'a std::string::String, &'a RawChoice);
    type IntoIter = Rev<linear_map::Iter<'a, std::string::String, RawChoice>>;

    fn into_iter(self) -> Self::IntoIter {
        self.choices.iter().rev()
    }
}

impl<'a> IntoIterator for &'a Choices {
    type Item = (&'a std::string::String, &'a String);
    type IntoIter = Rev<linear_map::Iter<'a, std::string::String, String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.choices.iter().rev()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Choices {
    choices: LinearMap<String, String>,
    #[serde(default)]
    pub timeout: f64,
}

impl Choices {
    pub fn new(choices: LinearMap<String, String>, timeout: f64) -> Self {
        Self { choices, timeout }
    }

    pub fn remove(&mut self, choice: &str) -> Option<String> {
        self.choices.remove(choice)
    }

    pub fn from(other: &mut Self) -> Result<Self> {
        Ok(Self {
            choices: LinearMap::move_values(&mut other.choices)?,
            timeout: other.timeout,
        })
    }

    pub fn get_valid(raw: &RawChoices, bookmark: &Bookmark) -> Result<Self> {
        let mut valid = Self {
            choices: LinearMap::default(),
            timeout: raw.timeout,
        };
        //  The current passage target.
        let mut passage = "";

        // Populate through valid choices and infer implicit passage targets.
        for (key, choice) in raw {
            match choice {
                // Populate top level choices.
                RawChoice::PassageName(Some(passage_name)) => {
                    passage = passage_name;
                    valid
                        .choices
                        .insert(key.to_string(), passage_name.to_string());
                }
                RawChoice::PassageName(None) => {
                    valid.choices.insert(key.to_string(), passage.to_string());
                }
                // Populate all choices are behind a true conditional.
                RawChoice::Conditional(conditional) => {
                    if !Value::from_conditional(key, bookmark)? {
                        continue;
                    }
                    for (choice_text, passage_name_opt) in conditional.iter().rev() {
                        if let Some(passage_name) = passage_name_opt {
                            passage = passage_name;
                            valid
                                .choices
                                .insert(choice_text.to_string(), passage_name.to_string());
                        } else {
                            valid.choices.insert(key.to_string(), passage.to_string());
                        }
                    }
                }
            }
        }
        Ok(valid)
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
                choice3: test1
                choice2: test2
                choice1:
                choice4: test3
        "#;

        let raw = RawChoices::from_yml(choices_str).unwrap();
        for raw_choice in &raw {
            println!("{:#?}", raw_choice);
        }

        let choices = Choices::get_valid(&raw, &bookmark).unwrap();
        for choice in &choices {
            println!("{:#?}", choice);
        }
    }
}
