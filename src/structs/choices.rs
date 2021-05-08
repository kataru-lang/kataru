use super::Bookmark;
use crate::{error::Result, Map, Value};
use linear_map::LinearMap;
use serde::{Deserialize, Serialize};

const EMPTY_STRING: &String = &String::new();

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

impl RawChoices {
    fn len(&self) -> usize {
        self.choices.len()
    }
}

impl<'a> IntoIterator for &'a RawChoices {
    type Item = (&'a String, &'a RawChoice);
    type IntoIter = linear_map::Iter<'a, String, RawChoice>;

    fn into_iter(self) -> Self::IntoIter {
        self.choices.iter()
    }
}

impl<'a> IntoIterator for &'a Choices {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.choices.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Choices {
    // List of choices matching the order of the raw chocies.
    pub choices: Vec<String>,
    #[serde(default)]
    pub timeout: f64,
}

impl Choices {
    fn push(&mut self, choice: &str) {
        self.choices.push(choice.to_string());
    }

    fn clear(&mut self) {
        self.choices.clear()
    }

    fn reserve(&mut self, additional: usize) {
        self.choices.reserve(additional)
    }

    fn reverse(&mut self) {
        self.choices.reverse()
    }

    /// Repopulates `self` with a list of all valid choices from `raw` in order.
    /// Also repopulates the `choice_to_passage` map which updated mappings.
    pub fn get_valid<'r>(
        &mut self,
        choice_to_passage: &mut Map<&'r str, &'r str>,
        raw: &'r RawChoices,
        bookmark: &Bookmark,
    ) -> Result<()> {
        // Reset structs.
        choice_to_passage.clear();
        choice_to_passage.reserve(raw.len());
        self.clear();
        self.reserve(raw.len());

        //  The current passage target.
        let mut passage: &String = &EMPTY_STRING;

        // Populate through valid choices and infer implicit passage targets.
        for (key, choice) in raw.into_iter().rev() {
            match choice {
                // Populate top level choices.
                RawChoice::PassageName(Some(passage_name)) => {
                    passage = passage_name;
                    self.push(key);
                    choice_to_passage.insert(key, passage);
                }
                RawChoice::PassageName(None) => {
                    self.push(key);
                    choice_to_passage.insert(key, passage);
                }
                // Populate all choices are behind a true conditional.
                RawChoice::Conditional(conditional) => {
                    if !Value::from_conditional(key, bookmark)? {
                        continue;
                    }
                    for (choice_text, passage_name_opt) in conditional.iter().rev() {
                        if let Some(passage_name) = passage_name_opt {
                            passage = passage_name;
                            self.push(choice_text);
                            choice_to_passage.insert(choice_text, passage);
                        } else {
                            self.push(choice_text);
                            choice_to_passage.insert(choice_text, passage);
                        }
                    }
                }
            }
        }

        // Since we iterated backwards for populating chocies, we must reverse to match order.
        self.reverse();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::FromYaml;

    impl FromYaml for RawChoices {}

    #[test]
    fn test_choices_order() {
        let bookmark = Bookmark::new(hashmap! {});
        let choices_str = r#"
            choices:
                c: C
                b: B
                a:
                d: D
        "#;

        let raw = RawChoices::from_yml(choices_str).unwrap();
        println!("{:#?}", raw);
        let mut choices = Choices::default();
        let mut choice_to_passage = Map::default();

        choices
            .get_valid(&mut choice_to_passage, &raw, &bookmark)
            .unwrap();
        assert_eq!(
            choices.choices,
            vec![
                "c".to_string(),
                "b".to_string(),
                "a".to_string(),
                "d".to_string()
            ]
        );
        assert_eq!(
            choice_to_passage,
            hashmap! {
                "c" => "C",
                "b" => "B",
                "a" => "D",
                "d" => "D",
            }
        );
        println!("{:#?}", choice_to_passage);
    }
}
