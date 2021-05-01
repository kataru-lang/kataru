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
            return Self::from(name, text, story, bookmark);
        }
        Ok(Self::default())
    }

    pub fn from(name: &str, text: &str, story: &Story, bookmark: &Bookmark) -> Result<Self> {
        let (attributes, text) = Self::extract_attr(&text, bookmark.namespace(), story)?;

        // For local characters, append the namespace to their name.
        let name = if bookmark.character_is_local(story, name) {
            format!("{}:{}", bookmark.namespace(), name)
        } else {
            name.to_string()
        };

        Ok(Self {
            name,
            text: replace_vars(&text, bookmark),
            attributes: attributes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Map, Section, GLOBAL};

    #[test]
    fn test_dialogue() {
        let namespace = GLOBAL.to_string();
        let story = btreemap! {
            GLOBAL.to_string() => Section {
                config: Config {
                    namespace,
                    attributes: btreemap! {
                        "attr".to_string() => None
                    },
                    ..Config::default()
                },
                passages: Map::new()
            }
        };
        let bookmark = Bookmark::new(btreemap! {
            GLOBAL.to_string() => Map::new()
        });
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
