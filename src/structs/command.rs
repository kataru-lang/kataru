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
        for value in self {
            return Ok(value);
        }

        Err(error!("Command was empty"))
    }

    /// Checks `story`'s config for default parameters for this command.
    /// Uses `bookmark` to lookup variable values.
    /// If the command is not found, returns None.
    /// If there are no parameters for the command, return reference to
    /// the static empty param map.
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

    /// Gets a qualified command name (prefixed with namespace if not global).
    fn get_qualified_command(
        story: &Story,
        bookmark: &Bookmark,
        character: &str,
        command_name: &str,
    ) -> Result<String> {
        let character = bookmark.qualified_character_name(story, character)?;
        Ok(format!("{}.{}", character, command_name))
    }

    /// Returns the (normalized_name, qualified_command) strings.
    /// If thhe command is prefixed by a character, returns $character.command in normalized name for lookup.
    /// IF the character is local, the qualified_command will have the namespace prepended.
    fn get_command_components(
        story: &Story,
        bookmark: &Bookmark,
        command_name: &str,
    ) -> Result<(String, String)> {
        let split: Vec<&str> = command_name.split(".").collect();

        // Handle character commands
        match split.as_slice() {
            [character, command_name] => Ok((
                format!("$character.{}", command_name),
                Self::get_qualified_command(story, bookmark, character, command_name)?,
            )),
            [command_name] => Ok((command_name.to_string(), command_name.to_string())),
            _ => return Err(error!("Commands can only contain one '.' delimeter.")),
        }
    }

    /// Get the vector of qualified commands with default parameters included.
    fn get_full_command(&self, story: &Story, bookmark: &Bookmark) -> Result<Command> {
        let (command_name, params) = self.get_first()?;
        let mut command = Command::default();
        let (normalized_name, qualified_command) =
            Self::get_command_components(story, bookmark, command_name)?;

        let default_params = RawCommand::get_default_params(story, bookmark, &normalized_name)?;
        // Merge params with their defaults.
        let mut merged_params = params.merge_params(default_params)?;

        // If the params have variable names, replace with variable value.
        for (_var, val) in merged_params.iter_mut() {
            val.eval_as_expr(bookmark)?;
        }

        command.name = qualified_command;
        command.params = merged_params;

        Ok(command)
    }
}

impl CommandGetters<PositionalParams> for PositionalCommand {}

impl CommandGetters<Params> for RawCommand {}
