use super::{extract_attr, Attributes, Bookmark, Map, Story};
use crate::error::{Error, Result};
use crate::vars::replace_vars;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dialogue {
    pub name: String,
    pub text: String,
    pub attributes: Attributes,
}

impl Dialogue {
    pub fn extract_attr(
        text: &str,
        namespace: &str,
        story: &Story,
    ) -> Result<(Attributes, String)> {
        match story.get(namespace) {
            Some(section) => extract_attr(text, &section.config.attributes),
            None => Err(error!(
                "No such namespace '{}', required for checking attributes in '{}'",
                namespace, text
            )),
        }
    }

    pub fn from_map(map: &Map<String, String>, story: &Story, bookmark: &Bookmark) -> Result<Self> {
        for (name, text) in map {
            let (attributes, text) =
                Self::extract_attr(&text, &bookmark.position.namespace, story)?;
            return Ok(Self {
                name: name.clone(),
                text: replace_vars(&text, bookmark),
                attributes: attributes,
            });
        }
        Ok(Self::default())
    }

    pub fn from(name: &str, text: &str, story: &Story, bookmark: &Bookmark) -> Result<Self> {
        let (attributes, text) = Self::extract_attr(&text, &bookmark.position.namespace, story)?;
        Ok(Self {
            name: name.to_string(),
            text: replace_vars(&text, bookmark),
            attributes: attributes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Map, Position, Section, State, GLOBAL};

    #[test]
    fn test_dialogue() {
        // let mut story;
        let namespace = GLOBAL.to_string();
        let story = btreemap! {
            GLOBAL.to_string() => Section {
                config: Config {
                    namespace,
                    commands: Map::new(),
                    state: State::new(),
                    attributes: btreemap! {
                        "attr".to_string() => None
                    },
                    characters: Map::new(),
                    on_passage: None
                },
                passages: Map::new()
            }
        };
        let bookmark = Bookmark {
            position: Position {
                namespace: GLOBAL.to_string(),
                passage: "".to_string(),
                line: 0,
            },
            state: btreemap! {
                GLOBAL.to_string() => Map::new()
            },
            stack: Vec::new(),
        };
        let dialogue_map =
            btreemap! {"Character".to_string() => "Text <attr>annotated</attr>.".to_string()};
        let dialogue = Dialogue::from_map(&dialogue_map, &story, &bookmark).unwrap();

        assert_eq!(
            dialogue,
            Dialogue {
                name: "Character".to_string(),
                text: "Text annotated.".to_string(),
                attributes: btreemap! {
                    "attr".to_string() => vec![5 as usize, 14]
                }
            }
        )
    }
}
