use linear_map::LinearMap;

use super::QualifiedName;
use crate::{traits::CopyMerge, Bookmark, Error, Map, Result, Story, Value};
use serde::{Deserialize, Serialize};

pub type Params = LinearMap<String, Value>;
pub type RawCommand = Map<String, Params>;

/// Public interface for a command.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Command {
    pub name: String,
    pub params: LinearMap<String, Value>,
}

pub type PositionalParams = Vec<Value>;
pub type PositionalCommand = Map<String, PositionalParams>;

lazy_static! {
    static ref EMPTY_PARAMS: Params = Params::default();
}

/// Trait for merging params with their defaults.
pub trait MergeParams {
    /// Merge parameters that have given values with `default_params`.
    fn merge_params(&self, default_params: &Params) -> Result<Params>;
}

impl MergeParams for PositionalParams {
    /// Merge parameters that have given values with `default_params`.
    fn merge_params(&self, default_params: &Params) -> Result<Params> {
        let mut merged_params = Params::new();
        let mut it = self.iter();
        for (param, default_value) in default_params {
            let value = if let Some(positional_value) = it.next() {
                positional_value.clone()
            } else {
                default_value.clone()
            };
            merged_params.insert(param.clone(), value);
        }
        Ok(merged_params)
    }
}

impl MergeParams for Params {
    fn merge_params(&self, default_params: &Params) -> Result<Params> {
        self.copy_merge(default_params)
    }
}

pub trait CommandGetters<ParamsT: MergeParams>: Sized
where
    for<'a> &'a Self: IntoIterator<Item = (&'a String, &'a ParamsT)>,
{
    /// Gets the first entry in the command map.
    /// Command is really a pairing, so the map should only have one value.
    fn get_first(&self) -> Result<(&String, &ParamsT)> {
        if let Some(value) = self.into_iter().next() {
            Ok(value)
        } else {
            Err(error!("Command was empty"))
        }
    }

    /// Checks `story`'s config for default parameters for this command.
    /// Uses `bookmark` to lookup variable values.
    /// If the command is not found, returns None.
    /// If there are no parameters for the command, return reference to
    /// the static empty param map.
    #[allow(dead_code)]
    fn get_default_params<'s>(
        story: &'s Story,
        bookmark: &Bookmark,
        command_name: &str,
    ) -> Result<&'s Params> {
        match story.params(&QualifiedName::from(bookmark.namespace(), command_name))? {
            Some(params) => Ok(params),
            None => Ok(&EMPTY_PARAMS),
        }
    }

    /// Builds a command with an optional character name.
    /// To build a normal command, pass `character_name = ""`.
    fn build_command_with_character(
        story: &Story,
        bookmark: &Bookmark,
        command_name: &str,
        params: &ParamsT,
        character_name: &str,
    ) -> Result<Command> {
        // Character commands need $character prepended before lookup in the story config.
        let normalized_name = if character_name.is_empty() {
            command_name.to_string()
        } else {
            format!("$character.{}", command_name)
        };
        let qname = QualifiedName::from(bookmark.namespace(), &normalized_name);
        let (resolved_namespace, _section, default_param_opt) = story.command(&qname)?;

        Ok(Command {
            name: if character_name.is_empty() {
                // Non-character commands need their qualified namespace prepended.
                // E.g. "namespace:command".
                qname.to_string(resolved_namespace)
            } else {
                // Character commands need their qualified character prepended.
                // E.g. "namespace:character.command".
                format!(
                    "{}.{}",
                    bookmark.qualified_character_name(story, character_name)?,
                    command_name
                )
            },
            params: match default_param_opt {
                Some(default_params) => params.merge_params(default_params)?,
                None => params.merge_params(&EMPTY_PARAMS)?,
            },
        })
    }

    /// Returns the (normalized_name, qualified_command) strings.
    /// If thhe command is prefixed by a character, returns $character.command in normalized name for lookup.
    /// If the character is local, the qualified_command will have the namespace prepended.
    fn build_init_command(
        story: &Story,
        bookmark: &Bookmark,
        command_name: &str,
        params: &ParamsT,
    ) -> Result<Command> {
        // Split on "." to identify character commands.
        let split: Vec<&str> = command_name.split(".").collect();
        let (character_name, command_name) = match split.as_slice() {
            [character_name, command_name] => (character_name, command_name),
            [command_name] => (&"", command_name),
            _ => {
                return Err(error!(
                    "Commands can only contain one '.' delimeter, but was '{}'",
                    command_name
                ))
            }
        };
        Self::build_command_with_character(story, bookmark, command_name, params, character_name)
    }

    /// Get the vector of qualified commands with default parameters included.
    fn build_command(&self, story: &Story, bookmark: &Bookmark) -> Result<Command> {
        let (command_name, params) = self.get_first()?;
        let mut command = Self::build_init_command(story, bookmark, command_name, params)?;

        // If the params have variable names, replace with variable value.
        for (_var, val) in command.params.iter_mut() {
            val.eval_as_expr(bookmark)?;
        }

        Ok(command)
    }
}

impl CommandGetters<PositionalParams> for PositionalCommand {}

impl CommandGetters<Params> for RawCommand {}
