use super::{CharacterData, Line, Map, Params, QualifiedName, Section};
use crate::{
    error::{Error, Result},
    traits::SaveYaml,
    traits::{FromMessagePack, FromYaml, Load, LoadYaml, Save, SaveMessagePack},
    LoadMessagePack, Merge, Value, GLOBAL,
};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub type Passage = Vec<Line>;

pub type Passages = Map<String, Passage>;

impl FromYaml for Passages {
    fn from_yml(text: &str) -> Result<Self> {
        // Avoid parsing whitespace only strings.
        if text.trim_start().is_empty() {
            return Ok(Self::new());
        }

        match serde_yaml::from_str(text) {
            Ok(config) => Ok(config),
            Err(e) => Err(error!("Invalid YAML for passages: {}", e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct Story {
    pub sections: Map<String, Section>,
}

impl<'a> Story {
    pub fn new() -> Self {
        Self {
            sections: Map::new(),
        }
    }

    /// Attempts to get a section containing the passage matching `qname`.
    /// First checks in the specified namespace, and falls back to root namespace if not found.
    ///
    /// Note that passage name could be:
    /// 1. a local name (unquallified), in which case namespace stays the same.
    /// 2. a qualified name pointing to another section, in which case we switch namespace.
    /// 3. a global name, in which we must changed namespace to root.
    pub fn section_for_passage(
        &'a self,
        qname: &mut QualifiedName,
    ) -> Result<(&'a Section, &'a Passage)> {
        // First try to find the section specified namespace.
        if let Some(section) = self.sections.get(&qname.namespace) {
            if let Some(passage) = section.passage(&qname.name) {
                // Case 2: name is not local, so switch namespace.
                return Ok((section, passage));
            }
        } else {
            return Err(error!("Invalid namespace '{}'", &qname.namespace));
        }

        // Fall back to try global namespace.
        if let Some(section) = self.sections.get(GLOBAL) {
            if let Some(passage) = section.passage(&qname.name) {
                // Case 3: passage could not be found in local/specified namespace, so switch to global.
                qname.namespace = GLOBAL.to_string();
                return Ok((section, passage));
            }
        } else {
            return Err(error!("No global namespace"));
        }

        // Return error if there is no passage name in either namespace.
        Err(error!(
            "Passage name '{}' could not be found in '{}' nor global namespace",
            qname.name, qname.namespace
        ))
    }

    pub fn character(&'a self, qname: &QualifiedName) -> Option<&'a Option<CharacterData>> {
        match self.sections.get(&qname.namespace)?.character(&qname.name) {
            Some(data) => Some(data),
            None => self.sections.get(GLOBAL)?.character(&qname.name),
        }
    }

    pub fn passage(&'a self, qname: &QualifiedName) -> Option<&'a Passage> {
        match self.sections.get(&qname.namespace)?.passage(&qname.name) {
            Some(data) => Some(data),
            None => self.sections.get(GLOBAL)?.passage(&qname.name),
        }
    }

    pub fn value(&'a self, qname: &QualifiedName) -> Option<&'a Value> {
        match self.sections.get(&qname.namespace)?.value(&qname.name) {
            Some(data) => Some(data),
            None => self.sections.get(GLOBAL)?.value(&qname.name),
        }
    }

    pub fn params(&'a self, qname: &QualifiedName) -> Option<&'a Option<Params>> {
        match self.sections.get(&qname.namespace)?.params(&qname.name) {
            Some(data) => Some(data),
            None => self.sections.get(GLOBAL)?.params(&qname.name),
        }
    }
}

impl<'a> FromMessagePack for Story {}

impl SaveYaml for Story {}
impl SaveMessagePack for Story {}
impl Save for Story {}
impl FromYaml for Story {}

impl LoadYaml for Story {
    /// Loads a story from a given directory.
    fn load_yml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut story = Self::new();
        let pattern: &str = &path
            .as_ref()
            .join("**/*.yml")
            .into_os_string()
            .into_string()
            .unwrap();
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                let mut section = Section::load_yml(path)?;
                let namespace = section.namespace();
                match story.sections.get_mut(namespace) {
                    Some(story_section) => {
                        story_section.merge(&mut section)?;
                    }
                    None => {
                        story.sections.insert(namespace.to_string(), section);
                    }
                };
            }
        }
        Ok(story)
    }
}

impl LoadMessagePack for Story {}
impl Load for Story {}
