use linear_map::LinearMap;

use crate::{traits::CopyMerge, Bookmark, Error, Map, Result, Story, StoryGetters, Value};

use super::QualifiedName;

pub type Params = LinearMap<String, Value>;
pub type Command = Map<String, Params>;

pub type PositionalParams = Vec<Value>;
pub type PositionalCommand = Map<String, PositionalParams>;

/// Trait for merging params with their defaults.
pub trait MergeParams {
    fn merge_params(&self, default_params: &Params) -> Result<Params>;
}

impl MergeParams for PositionalParams {
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
    fn get_first(&self) -> Result<(&String, &ParamsT)> {
        for value in self {
            return Ok(value);
        }

        Err(error!("Command was empty"))
    }

    fn get_default_params<'s>(
        story: &'s Story,
        bookmark: &Bookmark,
        command_name: &str,
    ) -> Option<&'s Params> {
        Some(
            story
                .params(&QualifiedName::from(
                    &bookmark.position.namespace,
                    command_name,
                ))?
                .as_ref()?,
        )
    }

    /// If `character` is local, then prepend the namespace to the character.command.
    fn get_qualified_command(
        story: &Story,
        bookmark: &Bookmark,
        character: &str,
        command_name: &str,
    ) -> String {
        // If currently in global namespace, don't bother checking.
        if bookmark.character_is_local(story, character) {
            format!(
                "{}:{}.{}",
                &bookmark.position.namespace, character, command_name
            )
        } else {
            format!("{}.{}", character, command_name)
        }
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
                Self::get_qualified_command(story, bookmark, character, command_name),
            )),
            [command_name] => Ok((command_name.to_string(), command_name.to_string())),
            _ => return Err(error!("Commands can only contain one '.' delimeter.")),
        }
    }

    /// Get the vector of qualified commands with default parameters included.
    fn get_full_command(&self, story: &Story, bookmark: &Bookmark) -> Result<Command> {
        let (command_name, params) = self.get_first()?;
        let mut full_command = Command::new();
        let (normalized_name, qualified_command) =
            Self::get_command_components(story, bookmark, command_name)?;

        if let Some(default_params) = Command::get_default_params(story, bookmark, &normalized_name)
        {
            // Merge params with their defaults.
            let mut merged_params = params.merge_params(default_params)?;

            // If the params have variable names, replace with variable value.
            for (_var, val) in merged_params.iter_mut() {
                val.eval_in_place(bookmark)?;
            }

            full_command.insert(qualified_command, merged_params);
        }

        Ok(full_command)
    }
}

impl CommandGetters<PositionalParams> for PositionalCommand {}

impl CommandGetters<Params> for Command {}
