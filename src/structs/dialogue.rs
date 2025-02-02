use super::{AttributeExtractor, Attributes, Bookmark, Map, Story};
use crate::error::Result;
use crate::vars::replace_vars;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dialogue {
    pub name: String,
    pub text: String,
    pub attributes: Attributes,
}

impl Dialogue {
    pub fn from_map(map: &Map<String, String>, story: &Story, bookmark: &Bookmark) -> Result<Self> {
        if let Some((name, text)) = map.iter().next() {
            return Self::from(name, text, story, bookmark);
        }
        Ok(Self::default())
    }

    pub fn from(name: &str, text: &str, story: &Story, bookmark: &Bookmark) -> Result<Self> {
        let (attributes, text) =
            AttributeExtractor::extract_attr(text, bookmark.namespace(), story)?;

        // For local characters, append the namespace to their name.
        let name = bookmark.qualified_character_name(story, name)?;

        Ok(Self {
            name,
            text: replace_vars(&text, bookmark),
            attributes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{structs::attributes::AttributedSpan, Config, Map, Section, GLOBAL};

    #[test]
    fn test_dialogue() {
        let namespace = GLOBAL.to_string();
        let story = Story::from(hashmap! {
            GLOBAL.to_string() => Section::new(
                Config {
                    namespace,
                    characters: hashmap! {
                        "Character".to_string() => None
                    },
                    attributes: hashmap! {
                        "attr".to_string() => None
                    },
                    ..Config::default()
                }
            )
        });
        let bookmark = Bookmark::new(hashmap! {
            GLOBAL.to_string() => Map::new()
        });
        let dialogue_map =
            hashmap! {"Character".to_string() => "Text <attr>annotated</attr>.".to_string()};
        let dialogue = Dialogue::from_map(&dialogue_map, &story, &bookmark).unwrap();

        assert_eq!(
            dialogue,
            Dialogue {
                name: "Character".to_string(),
                text: "Text annotated.".to_string(),
                attributes: vec![AttributedSpan {
                    start: 5,
                    end: 14,
                    params: hashmap! {
                        "attr".to_string() => None
                    }
                }]
            }
        )
    }
}
