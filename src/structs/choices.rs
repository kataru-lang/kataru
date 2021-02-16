use super::{Bookmark, Conditional, Map};
use crate::error::Result;
use crate::traits::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Choice {
    Conditional(Map<String, String>),
    PassageName(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Choices {
    pub choices: Map<String, Choice>,
    #[serde(default)]
    pub timeout: f64,
}

impl Choices {
    pub fn get_valid(&self, bookmark: &Bookmark) -> Result<Self> {
        let mut valid = Self {
            choices: Map::default(),
            timeout: self.timeout,
        };
        for (key, choice) in &self.choices {
            match choice {
                Choice::PassageName(passage_name) => {
                    valid.choices.insert(
                        key.to_string(),
                        Choice::PassageName(passage_name.to_string()),
                    );
                }
                Choice::Conditional(conditional) => {
                    for (choice_text, passage_name) in conditional {
                        if !Conditional::from_str(key)?.eval(&bookmark)? {
                            continue;
                        }
                        valid.choices.insert(
                            choice_text.to_string(),
                            Choice::PassageName(passage_name.to_string()),
                        );
                    }
                }
            }
        }
        Ok(valid)
    }
}
