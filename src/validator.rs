use crate::{
    error::{Error, Result},
    structs::{
        Branches, Cmd, Comparator, Conditional, Dialogue, Line, Map, Operator, Params, Passage,
        Passages, QualifiedName, RawChoice, RawChoices, State, StateMod, Story, StoryGetters,
        Value, GLOBAL,
    },
    traits::FromStr,
};

pub struct Validator<'a> {
    namespace: &'a str,
    passage: &'a str,
    story: &'a Story,
}

impl<'a> Validator<'a> {
    pub fn new(story: &'a Story) -> Self {
        Self {
            namespace: GLOBAL,
            passage: "",
            story,
        }
    }

    fn validate_text(&self, text: &str) -> Result<()> {
        Dialogue::extract_attr(text, self.namespace, self.story)?;
        Ok(())
    }

    fn validate_character(&self, name: &str) -> Result<()> {
        if self
            .story
            .character(&QualifiedName::from(self.namespace, name))
            .is_none()
        {
            return Err(error!("Undefined character name {}", name));
        }
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
    fn validate_conditional(&self, expression: &str) -> Result<()> {
        let cond = Conditional::from_str(expression)?;
        let value = self.validate_var(cond.var)?;
        Self::validate_cmp(&cond.val, value, cond.cmp)
    }

    /// Validates conditional branches.
    fn validate_branches(&self, branches: &Branches) -> Result<()> {
        for (expression, lines) in branches {
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

    fn validate_command(&self, namespace: &str, command_name: &str, params: &Params) -> Result<()> {
        match self
            .story
            .params(&QualifiedName::from(namespace, &command_name))
        {
            None => {
                if namespace == GLOBAL {
                    Err(error!("No such command '{}'.", command_name))
                } else {
                    self.validate_command(GLOBAL, command_name, params)
                }
            }
            Some(Some(config_params)) => Self::validate_params(command_name, params, config_params),
            Some(None) => Ok(()),
        }
    }

    /// Validates a list of commands in the Cmd object.
    fn validate_cmd(&self, cmd: &Cmd) -> Result<()> {
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

            self.validate_command(&self.namespace, &command_name, params)?;
        }
        Ok(())
    }

    /// Validates a list of commands.
    fn validate_cmds(&self, cmds: &Vec<Cmd>) -> Result<()> {
        for cmd in cmds {
            self.validate_cmd(cmd)?
        }
        Ok(())
    }

    /// Validates a line of dialogue.
    fn validate_line(&self, line: &Line) -> Result<()> {
        match &line {
            Line::RawDialogue(dialogue) => self.validate_dialogue(dialogue),
            Line::Branches(cond) => self.validate_branches(cond),
            Line::RawChoices(choices) => self.validate_choices(choices),
            Line::Call(call) => self.validate_goto(&call.passage),
            Line::SetCmd(cmd) => self.validate_state(&cmd.set),
            Line::Commands(cmds) => self.validate_cmds(&cmds),
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
    fn validate_op(v1: &Value, v2: &Value, op: Operator) -> Result<()> {
        match op {
            Operator::SET => {
                if v1.same_type(v2) {
                    Ok(())
                } else {
                    Err(error!(
                        "Operators require operands of the same type, not {:?} and {:?}",
                        v1, v2
                    ))
                }
            }
            Operator::ADD | Operator::SUB => match (v1, v2) {
                (Value::Number(_), Value::Number(_)) => Ok(()),
                _ => Err(error!(
                    "Comparators '+,-' can only be used on two numbers, not {:?} and {:?}.",
                    v1, v2
                )),
            },
        }
    }

    /// Validates an comparator on a given value.
    /// Any value supports assignment, but only Numbers can be added or subtracted.
    fn validate_cmp(v1: &Value, v2: &Value, cmp: Comparator) -> Result<()> {
        match cmp {
            Comparator::EQ | Comparator::NEQ => {
                if v1.same_type(v2) {
                    Ok(())
                } else {
                    Err(error!(
                        "Comparisons require values of the same type, not {:?} and {:?}",
                        v1, v2
                    ))
                }
            }
            Comparator::LT | Comparator::LEQ | Comparator::GT | Comparator::GEQ => match (v1, v2) {
                (Value::Number(_), Value::Number(_)) => Ok(()),
                _ => Err(error!(
                "Comparators '>,>=,<,<=' can only be used between two numbers, not {:?} and {:?}.",
                v1,
                v2
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
                if let Some(value) = self
                    .story
                    .value(&QualifiedName::from(self.namespace, &passage_var))
                {
                    self.validate_goto(prefix)?;
                    return Ok(value);
                }

                // Then check character variables.
                let character_var = format!("$character.{}", suffix);
                if let Some(value) = self
                    .story
                    .value(&QualifiedName::from(self.namespace, &character_var))
                {
                    self.validate_character(prefix)?;
                    return Ok(value);
                }

                Err(error!(
                    "Variable '{}' did not match any character or passage variables.",
                    var
                ))
            }
            [var] => {
                if let Some(value) = self.story.value(&QualifiedName::from(self.namespace, &var)) {
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
            let smod = StateMod::from_str(key)?;
            let state_value = self.validate_var(smod.var)?;
            Self::validate_op(state_value, value, smod.op)?;
        }
        Ok(())
    }

    fn validate_goto(&self, passage_name: &str) -> Result<()> {
        match self
            .story
            .passage(&QualifiedName::from(self.namespace, passage_name))
        {
            None => Err(error!(
                "Passage name '{}' was not defined in the story.",
                passage_name
            )),
            Some(_) => Ok(()),
        }
    }

    /// Validates that the story contains the referenced passage.
    fn validate_choices(&self, choices: &RawChoices) -> Result<()> {
        for (key, choice) in &choices.choices {
            match choice {
                RawChoice::PassageName(Some(passage_name)) => self.validate_goto(&passage_name)?,
                RawChoice::Conditional(conditional) => {
                    for (_choice_name, passage_name_opt) in conditional {
                        self.validate_conditional(key)?;
                        if let Some(passage_name) = passage_name_opt {
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
            self.passage = passage_name;
            if let Err(e) = self.validate_passage(passage) {
                return Err(error!(
                    "Passage '{}:{}' {}",
                    self.namespace, passage_name, e
                ));
            }
        }
        Ok(())
    }

    /// Validates an entire story for valid passage references, HTML, conditionals.
    pub fn validate(&mut self) -> Result<()> {
        for (namespace, namespace_val) in self.story {
            self.namespace = namespace;
            self.validate_passages(&namespace_val.passages)?;
        }
        Ok(())
    }
}
