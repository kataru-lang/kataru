use super::{CharacterData, Line, Map, Params, QualifiedName, Section, Value};
use crate::error::ParseError;
use crate::traits::{Deserializable, Loadable, Mergeable, Parsable};
use glob::glob;
use std::io;
use std::path::Path;

pub type Passage = Vec<Line>;

pub type Passages = Map<String, Passage>;

impl Parsable<'_> for Passages {
    fn parse(text: &str) -> Result<Self, ParseError> {
        // Avoid parsing whitespace only strings.
        if text.trim_start().is_empty() {
            return Ok(Self::new());
        }

        match serde_yaml::from_str(text) {
            Ok(config) => Ok(config),
            Err(e) => Err(perror!("{}", e)),
        }
    }
}

pub type Story = Map<String, Section>;

/// Each story getter returns an Option reference if the name is found.
/// Also returns a boolean flag that is true if the name was found in root namespace.
pub trait StoryGetters<'a> {
    fn character(&'a self, qname: &QualifiedName) -> Option<&'a CharacterData>;
    fn passage(&'a self, qname: &QualifiedName) -> Option<&'a Passage>;
    fn value(&'a self, qname: &QualifiedName) -> Option<&'a Value>;
    fn params(&'a self, qname: &QualifiedName) -> Option<&'a Option<Params>>;
}

impl<'a> StoryGetters<'a> for Story {
    fn character(&'a self, qname: &QualifiedName) -> Option<&'a CharacterData> {
        match self.get(&qname.namespace)?.character(&qname.name) {
            Some(data) => Some(data),
            None => self.get("")?.character(&qname.name),
        }
    }

    fn passage(&'a self, qname: &QualifiedName) -> Option<&'a Passage> {
        match self.get(&qname.namespace)?.passage(&qname.name) {
            Some(data) => Some(data),
            None => self.get("")?.passage(&qname.name),
        }
    }

    fn value(&'a self, qname: &QualifiedName) -> Option<&'a Value> {
        match self.get(&qname.namespace)?.value(&qname.name) {
            Some(data) => Some(data),
            None => self.get("")?.value(&qname.name),
        }
    }

    fn params(&'a self, qname: &QualifiedName) -> Option<&'a Option<Params>> {
        match self.get(&qname.namespace)?.params(&qname.name) {
            Some(data) => Some(data),
            None => self.get("")?.params(&qname.name),
        }
    }
}

impl Deserializable for Story {
    fn deserialize(bytes: &[u8]) -> Self {
        rmp_serde::from_slice(bytes).unwrap()
    }
}

impl Loadable for Story {
    /// Loads a story from a given directory.
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut story = Self::new();
        let pattern: &str = &path
            .as_ref()
            .join("**/*.yml")
            .into_os_string()
            .into_string()
            .unwrap();
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                let mut section = Section::load(path).unwrap();
                let namespace = section.config.namespace.clone();
                match story.get_mut(&namespace) {
                    Some(story_section) => {
                        story_section.merge(&mut section).unwrap();
                    }
                    None => {
                        story.insert(namespace, section);
                    }
                };
            }
        }
        Ok(story)
    }
}
