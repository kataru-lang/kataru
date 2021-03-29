use linear_map::LinearMap;

use crate::{traits::CopyMerge, Bookmark, Error, Map, Result, Story, StoryGetters, Value};

use super::QualifiedName;

pub type Params = LinearMap<String, Value>;
pub type Command = Map<String, Params>;

pub type PositionalParams = serde_yaml::Mapping;
pub type PositionalCommand = Map<String, PositionalParams>;

pub trait CommandGetters: Sized {
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
    fn get_full_command(story: &Story, bookmark: &Bookmark, command: &Self) -> Result<Self>;

    /// Get the qualified command with the parameters infered by their positions.
    fn get_full_positional_command(
        story: &Story,
        bookmark: &Bookmark,
        command: &PositionalCommand,
    ) -> Result<Self>;
}

impl CommandGetters for Command {
    /// Get the vector of qualified commands with default parameters included.
    fn get_full_command(story: &Story, bookmark: &Bookmark, command: &Self) -> Result<Self> {
        for (command_name, params) in command {
            let mut cmd = Command::new();
            let mut merged_params = Params::new();

            let (normalized_name, qualified_command) =
                Self::get_command_components(story, bookmark, command_name)?;

            if let Some(default_params) =
                Command::get_default_params(story, bookmark, &normalized_name)
            {
                merged_params = params.copy_merge(default_params)?;

                // If the params have variable names, replace with variable value.
                for (_var, val) in merged_params.iter_mut() {
                    println!("Eval in place {:?}", val);
                    val.eval_in_place(bookmark)?;
                }
            }

            cmd.insert(qualified_command, merged_params);
            return Ok(cmd);
        }
        Err(error!("Empty command."))
    }

    /// Get the qualified command with the parameters infered by their positions.
    fn get_full_positional_command(
        story: &Story,
        bookmark: &Bookmark,
        command: &PositionalCommand,
    ) -> Result<Self> {
        for (command_name, params) in command {
            let mut full_command = Command::new();
            let mut merged_params = Params::new();
            let (normalized_name, qualified_command) =
                Self::get_command_components(story, bookmark, command_name)?;

            // Collect all values from the given positional params.
            let mut values = Vec::new();
            values.reserve(params.len());
            for (yml_value, _null) in params {
                values.push(Value::from_yml(yml_value.clone())?);
            }
            values.reverse();

            if let Some(default_params) =
                Command::get_default_params(story, bookmark, &normalized_name)
            {
                for (param, default_value) in default_params {
                    let value = if let Some(positional_value) = values.pop() {
                        positional_value
                    } else {
                        default_value.clone()
                    };
                    merged_params.insert(param.clone(), value);
                }
            }

            full_command.insert(qualified_command, merged_params);
            return Ok(full_command);
        }
        Err(error!("Empty command."))
    }
}
