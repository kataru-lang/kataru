use crate::{
    error::{Error, Result},
    structs::{
        AssignOperator, Branches, ChoiceTarget, Dialogue, Map, Params, Passage, Passages,
        QualifiedName, RawChoice, RawChoices, RawCommand, RawLine, State, StateMod, Story,
        StoryGetters,
    },
    traits::FromStr,
    Bookmark, Value,
};

pub struct Validator<'a> {
    story: &'a Story,
    bookmark: &'a mut Bookmark,
}

impl<'a> Validator<'a> {
    pub fn new(story: &'a Story, bookmark: &'a mut Bookmark) -> Self {
        Self { story, bookmark }
    }

    fn validate_text(&self, text: &str) -> Result<()> {
        Dialogue::extract_attr(text, self.bookmark.namespace(), self.story)?;
        Ok(())
    }

    fn validate_character(&self, name: &str) -> Result<()> {
        self.story
            .character(&QualifiedName::from(self.bookmark.namespace(), name))?;
        Ok(())
    }

    /// Validate that the dialogue contains valid text and configured characters only.
    fn validate_dialogue(&self, dialogue: &Map<String, String>) -> Result<()> {
        for (name, text) in dialogue {
            self.validate_character(&name)?;
            self.validate_text(&text)?;
        }
        Ok(())
    }

    /// Validates a conditional statement.
    fn validate_conditional(&self, expr: &str) -> Result<()> {
        Value::from_conditional(expr, self.bookmark)?;
        Ok(())
    }

    /// Validates conditional branches.
    fn validate_branches(&self, branches: &Branches) -> Result<()> {
        for (expression, lines) in &branches.exprs {
            if expression != "else" {
                self.validate_conditional(expression)?;
            }
            self.validate_passage(lines)?;
        }
        Ok(())
    }

    /// Validates parameters for a function call.
    fn validate_params(command_name: &str, params: &Params, config_params: &Params) -> Result<()> {
        for (param, _val) in params {
            if !config_params.contains_key(param) {
                return Err(error!(
                    "No such parameter '{}' for command '{}'",
                    param, command_name
                ));
            }
        }
        Ok(())
    }

    fn validate_namespace_command(
        &self,
        namespace: &str,
        command_name: &str,
        params: &Params,
    ) -> Result<()> {
        match self
            .story
            .params(&QualifiedName::from(namespace, &command_name))?
        {
            Some(config_params) => Self::validate_params(command_name, params, config_params),
            None => Ok(()),
        }
    }

    /// Validates a list of commands in the Cmd object.
    fn validate_command(&self, cmd: &RawCommand) -> Result<()> {
        for (command, params) in cmd {
            let split: Vec<&str> = command.split(".").collect();
            let command_name = match split.as_slice() {
                [character, command] => {
                    self.validate_character(&character)?;
                    format!("$character.{}", command)
                }
                [command] => command.to_string(),
                _ => return Err(error!("Commands can only contain one '.' delimeter.")),
            };

            self.validate_namespace_command(&self.bookmark.namespace(), &command_name, params)?;
        }
        Ok(())
    }

    /// Validates a line of dialogue.
    fn validate_line(&self, line: &RawLine) -> Result<()> {
        match &line {
            RawLine::Dialogue(dialogue) => self.validate_dialogue(dialogue),
            RawLine::Branches(branches) => self.validate_branches(branches),
            RawLine::Choices(choices) => self.validate_choices(choices),
            RawLine::Call(call) => self.validate_goto(&call.passage),
            RawLine::SetCommand(set_command) => self.validate_state(&set_command.set),
            RawLine::Command(command) => self.validate_command(&command),
            _ => Ok(()),
        }
    }

    /// Validates that a line (either text or dialogue) has valid HTML and valid speakers.
    fn validate_passage(&self, lines: &Passage) -> Result<()> {
        for (i, line) in lines.iter().enumerate() {
            if let Err(e) = self.validate_line(line) {
                return Err(error!("Line {}: {}", i + 1, e));
            }
        }
        Ok(())
    }

    /// Validates an operator on a given value.
    /// Any value supports assignment, but only Numbers can be added or subtracted.
    fn validate_assign(v1: &Value, v2: &Value, op: AssignOperator) -> Result<()> {
        match op {
            AssignOperator::None => {
                if v1.same_type(v2) {
                    Ok(())
                } else {
                    Err(error!(
                        "Operators require operands of the same type, not {:?} and {:?}",
                        v1, v2
                    ))
                }
            }
            AssignOperator::Add | AssignOperator::Sub => match (v1, v2) {
                (Value::Number(_), Value::Number(_)) => Ok(()),
                _ => Err(error!(
                    "Comparators '+,-' can only be used on two numbers, not {:?} and {:?}.",
                    v1, v2
                )),
            },
        }
    }
    /// Validates a variable and returns a reference to it's value.
    fn validate_var(&self, var: &str) -> Result<&Value> {
        let split: Vec<&str> = var.split(".").collect();
        match split.as_slice() {
            [prefix, suffix] => {
                // First check passage variables.
                let passage_var = format!("$passage.{}", suffix);
                if let Ok(value) = self.story.value(&QualifiedName::from(
                    self.bookmark.namespace(),
                    &passage_var,
                )) {
                    self.validate_goto(prefix)?;
                    return Ok(value);
                }

                // Then check character variables.
                let character_var = format!("$character.{}", suffix);
                if let Ok(value) = self.story.value(&QualifiedName::from(
                    self.bookmark.namespace(),
                    &character_var,
                )) {
                    self.validate_character(prefix)?;
                    return Ok(value);
                }

                Err(error!(
                    "Variable '{}' did not match any character or passage variables.",
                    var
                ))
            }
            [var] => {
                if let Ok(value) = self
                    .story
                    .value(&QualifiedName::from(self.bookmark.namespace(), &var))
                {
                    Ok(value)
                } else {
                    Err(error!("Variable '{}' was undefined.", var))
                }
            }
            _ => Err(error!("Variables can only contain one '.' delimeter.")),
        }
    }

    /// Validates the state only contains configured keys.
    fn validate_state(&self, state: &State) -> Result<()> {
        for (key, value) in state {
            let mut value = value.clone();
            value.eval_as_expr(self.bookmark)?;
            let smod = StateMod::from_str(key)?;
            let state_value = self.validate_var(smod.var)?;
            Self::validate_assign(state_value, &value, smod.op)?;
        }
        Ok(())
    }

    fn validate_goto(&self, passage_name: &str) -> Result<()> {
        self.story.passage(&QualifiedName::from(
            &self.bookmark.namespace(),
            passage_name,
        ))?;
        Ok(())
    }

    /// Validates that the story contains the referenced passage.
    fn validate_choices(&self, choices: &RawChoices) -> Result<()> {
        for (key, choice) in choices {
            match choice {
                RawChoice::Target(ChoiceTarget::PassageName(passage_name)) => {
                    self.validate_goto(&passage_name)?
                }
                RawChoice::Conditional(conditional) => {
                    for (_choice_name, passage_name_opt) in conditional {
                        self.validate_conditional(key)?;
                        if let ChoiceTarget::PassageName(passage_name) = passage_name_opt {
                            self.validate_goto(&passage_name)?;
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn validate_passages(&mut self, passages: &'a Passages) -> Result<()> {
        for (passage_name, passage) in passages {
            self.bookmark.set_passage(passage_name.to_string());
            if let Err(e) = self.validate_passage(passage) {
                return Err(error!(
                    "Passage '{}:{}' {}",
                    self.bookmark.namespace(),
                    passage_name,
                    e
                ));
            }
        }
        Ok(())
    }

    /// Validates an entire story for valid passage references, HTML, conditionals.
    pub fn validate(&mut self) -> Result<()> {
        let original_position = self.bookmark.position().clone();
        for (namespace, namespace_val) in self.story {
            self.bookmark.set_namespace(namespace.to_string());
            self.validate_passages(&namespace_val.passages)?;
        }
        self.bookmark.set_position(original_position);
        Ok(())
    }
}
