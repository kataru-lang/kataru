use super::attributes::AttributeConfig;
use super::{CharacterData, Map, Params, QualifiedName, RawLine, Section};
use crate::error::{Error, Result};
use crate::traits::SaveYaml;
use crate::{
    traits::{FromMessagePack, FromYaml, Load, LoadYaml, Merge, Save, SaveMessagePack},
    LoadMessagePack,
};
use crate::{Bookmark, SetCommand, Value};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::{fmt, path::Path};

pub type Passage = Vec<RawLine>;
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

/// Represents the story, which is a map of namespaces to their sections.
#[derive(Debug, Deserialize, Default, Serialize)]
pub struct Story {
    #[serde(flatten)]
    pub sections: Map<String, Section>,
}
impl<'a> Story {
    /// Construct a story with an empty map of sections.
    pub fn new() -> Self {
        Self {
            sections: Map::new(),
        }
    }

    /// Iterates over possible resolutions of the identifier.
    /// Returns None if any of the namespaces don't exist or the identifier could not be found.
    fn resolve<T>(
        &'a self,
        qname: &QualifiedName,
        getter: fn(&'a Section, &str) -> Option<T>,
    ) -> Result<T> {
        let (_namespace, _section, data) = self.resolve_with_section(qname, getter)?;
        Ok(data)
    }

    /// Iterates over possible resolutions of the identifier.
    /// Returns None if any of the namespaces don't exist or the identifier could not be found.
    fn resolve_with_section<'n, T>(
        &'a self,
        qname: &'n QualifiedName,
        getter: fn(&'a Section, &'n str) -> Option<T>,
    ) -> Result<(&'n str, &'a Section, T)> {
        for namespace in qname.resolve() {
            // println!("Resolving '{}' in namespace '{}'", qname.name, namespace);
            if let Some(section) = self.sections.get(namespace) {
                if let Some(data) = getter(section, qname.name) {
                    return Ok((namespace, section, data));
                }
            } else {
                return Err(error!("Namespace '{}' does not exist", namespace));
            }
        }
        Err(error!(
            "Identifier '{}' was not found in any namespaces.",
            qname.name
        ))
    }

    /// Applies set commands
    pub fn apply_set_commands(
        &'a self,
        getter: fn(&'a Section) -> &Option<SetCommand>,
        bookmark: &mut Bookmark,
    ) -> Result<()> {
        let mut set_commands: Vec<&SetCommand> = Vec::new();

        // Collect all set commands to run.
        let qname = QualifiedName::from(bookmark.namespace(), "");
        for namespace in qname.resolve() {
            if let Some(section) = self.sections.get(namespace) {
                if let Some(set_cmd) = getter(section) {
                    set_commands.push(&set_cmd);
                }
            } else {
                return Err(error!("Invalid namespace '{}'", namespace));
            }
        }
        // Apply all  set commands to bookmark.
        for set_command in set_commands {
            bookmark.set_state(&set_command.set)?;
        }
        Ok(())
    }

    /// Gets character data and the enclosing section by resolving `qname`.
    pub fn character<'n>(
        &'a self,
        qname: &'n QualifiedName,
    ) -> Result<(&'n str, &'a Section, &'a Option<CharacterData>)> {
        match self.resolve_with_section(qname, |section, name| section.character(name)) {
            Ok((namespace, section, data)) => Ok((namespace, section, data)),
            Err(e) => Err(error!("Invalid character: {}", e)),
        }
    }

    /// Gets a value by resolving `qname`.
    pub fn value(&'a self, qname: &QualifiedName) -> Result<&'a Value> {
        match self.resolve(qname, |section, name| section.value(name)) {
            Ok(data) => Ok(data),
            Err(e) => Err(error!("Invalid variable: {}", e)),
        }
    }
    /// Gets the params for a command by resolving `qname`.
    pub fn params(&'a self, qname: &QualifiedName) -> Result<&'a Option<Params>> {
        match self.resolve(qname, |section, name| section.params(name)) {
            Ok(data) => Ok(data),
            Err(e) => Err(error!("Invalid command: {}", e)),
        }
    }

    /// Gets a passage by resolving `qname`.
    pub fn passage<'n>(
        &'a self,
        qname: &'n QualifiedName,
    ) -> Result<(&'n str, &'a Section, &'a Passage)> {
        match self.resolve_with_section(qname, |section, name| section.passage(name)) {
            Ok(data) => Ok(data),
            Err(e) => Err(error!("Invalid passage: {}", e)),
        }
    }

    /// Gets an attribute by resolving `qname`.
    pub fn attribute<'n>(
        &'a self,
        qname: &'n QualifiedName,
    ) -> Result<&'a Option<AttributeConfig>> {
        match self.resolve(qname, |section, name| section.attribute(name)) {
            Ok(data) => Ok(data),
            Err(e) => Err(error!("Invalid attribute: {}", e)),
        }
    }
}
impl From<Map<String, Section>> for Story {
    fn from(sections: Map<String, Section>) -> Self {
        Self { sections }
    }
}
impl<'a> FromMessagePack for Story {}
impl SaveYaml for Story {}
impl SaveMessagePack for Story {}
impl Save for Story {}
impl FromYaml for Story {}

fn load_section<P: AsRef<Path> + fmt::Debug>(story: &mut Story, section_path: P) -> Result<()> {
    let mut section = Section::load_yml(section_path)?;
    let namespace = section.namespace();
    match story.sections.get_mut(namespace) {
        Some(story_section) => {
            story_section.merge(&mut section)?;
        }
        None => {
            story.sections.insert(namespace.to_string(), section);
        }
    };
    Ok(())
}

impl LoadYaml for Story {
    /// Loads a story from a given directory or YAML file.
    fn load_yml<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<Self> {
        let mut story = Self::new();

        // Handle loading a single path story.
        if path.as_ref().is_file() {
            return match Self::load_string(path) {
                Ok(source) => Self::from_yml(&source),
                Err(e) => Err(error!("Error loading YAML: {}", e)),
            };
        }

        let pattern: &str = &path
            .as_ref()
            .join("**/*.yml")
            .into_os_string()
            .into_string()
            .unwrap();
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                load_section(&mut story, path)?;
            }
        }
        Ok(story)
    }
}

impl LoadMessagePack for Story {}
impl Load for Story {}
