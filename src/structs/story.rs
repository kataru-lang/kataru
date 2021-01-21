use super::{CharacterData, Line, Map, Params, Section, Value};
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
    fn character(&'a self, namespace: &str, name: &str) -> (Option<&'a CharacterData>, bool);
    fn passage(&'a self, namespace: &str, name: &str) -> (Option<&'a Passage>, bool);
    fn state(&'a self, namespace: &str, name: &str) -> (Option<&'a Value>, bool);
    fn cmd(&'a self, namespace: &str, name: &str) -> (Option<&'a Params>, bool);
}

impl<'a> StoryGetters<'a> for Story {
    fn character(&'a self, namespace: &str, name: &str) -> (Option<&'a CharacterData>, bool) {
        let (full_namespace, base_name) = resolve_namespace(namespace, name);
        get_from(
            &self.get(full_namespace).unwrap().config.characters,
            &self.get("").unwrap().config.characters,
            base_name,
        )
    }

    fn passage(&'a self, namespace: &str, name: &str) -> (Option<&'a Passage>, bool) {
        let (full_namespace, base_name) = resolve_namespace(namespace, name);
        get_from(
            &self.get(full_namespace).unwrap().passages,
            &self.get("").unwrap().passages,
            base_name,
        )
    }

    fn state(&'a self, namespace: &str, name: &str) -> (Option<&'a Value>, bool) {
        let (full_namespace, base_name) = resolve_namespace(namespace, name);
        get_from(
            &self.get(full_namespace).unwrap().config.state,
            &self.get("").unwrap().config.state,
            base_name,
        )
    }

    fn cmd(&'a self, namespace: &str, name: &str) -> (Option<&'a Params>, bool) {
        let (full_namespace, base_name) = resolve_namespace(namespace, name);
        get_from(
            &self.get(full_namespace).unwrap().config.cmds,
            &self.get("").unwrap().config.cmds,
            base_name,
        )
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
        println!("Loading story... {:?}", path.as_ref());
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

pub fn get_from<'a, T>(
    map: &'a Map<String, T>,
    fallback: &'a Map<String, T>,
    name: &str,
) -> (Option<&'a T>, bool) {
    match map.get(name) {
        Some(data) => (Some(data), false),
        None => (fallback.get(name), true),
    }
}

/// Gets the namespace if indicated by `name`.
/// Defaults to using `namespace`.
pub fn resolve_namespace<'a>(namespace: &'a str, name: &'a str) -> (&'a str, &'a str) {
    let split: Vec<&str> = name.rsplitn(2, ":").collect();
    match split.as_slice() {
        [split_name, explicit_namespace] => (explicit_namespace, split_name),
        _ => (namespace, name),
    }
}
