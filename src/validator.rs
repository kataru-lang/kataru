use crate::structs::{
    Branches, Choice, Choices, Cmd, Comparator, Conditional, Line, Map, Operator, Params, Passage,
    Passages, QualifiedName, State, StateMod, Story, StoryGetters, Value,
};
use crate::{error::Error, traits::FromStr};
use html_parser::Dom;

pub struct Validator<'a> {
    namespace: &'a str,
    story: &'a Story,
}

impl<'a> Validator<'a> {
    pub fn new(story: &'a Story) -> Self {
        Self {
            namespace: "",
            story,
        }
    }

    /// Validate text to guarantee valid HTML.
    #[allow(dead_code)]
    fn validate_text(text: &str) -> Result<(), Error> {
        match Dom::parse(text) {
            Err(e) => Err(error!("Text error: {}", e)),
            Ok(_) => Ok(()),
        }
    }

    /// Validate that the dialogue contains valid text and configured characters only.
    fn validate_dialogue(&self, dialogue: &Map<String, String>) -> Result<(), Error> {
        for (name, _text) in dialogue {
            if self
                .story
                .character(&QualifiedName::from(self.namespace, name))
                .is_none()
            {
                return Err(error!("Undefined character name {}", name));
            }
            // Self::validate_text(&text)?;
        }
        Ok(())
    }

    /// Validates a conditional statement.
    fn validate_conditional(&self, expression: &str) -> Result<(), Error> {
        let cond = Conditional::from_str(expression)?;
        let value = self.validate_var(cond.var)?;
        Self::validate_cmp(&cond.val, value, cond.cmp)
    }

    /// Validates conditional branches.
    fn validate_branches(&self, branches: &Branches) -> Result<(), Error> {
        for (expression, lines) in branches {
            if expression != "else" {
                self.validate_conditional(expression)?;
            }
            self.validate_passage(lines)?;
        }
        Ok(())
    }

    /// Validates parameters for a function call.
    fn validate_params(
        command: &str,
        params: &Params,
        config_params: &Params,
    ) -> Result<(), Error> {
        for (param, _val) in params {
            if !config_params.contains_key(param) {
                return Err(error!(
                    "No such parameter '{}' for command '{}'",
                    param, command
                ));
            }
        }
        Ok(())
    }

    /// Validates a command.
    fn validate_cmd(&self, cmd: &Cmd) -> Result<(), Error> {
        for (command, params) in cmd {
            match self
                .story
                .params(&QualifiedName::from(self.namespace, command))
            {
                None => Err(error!("No such command '{}'.", command)),
                Some(Some(config_params)) => Self::validate_params(command, params, config_params),
                Some(None) => Ok(()),
            }?
        }
        Ok(())
    }

    /// Validates a list of commands.
    fn validate_cmds(&self, cmds: &Vec<Cmd>) -> Result<(), Error> {
        for cmd in cmds {
            self.validate_cmd(cmd)?
        }
        Ok(())
    }

    /// Validates a line of dialogue.
    fn validate_line(&self, line: &Line) -> Result<(), Error> {
        match &line {
            Line::_Dialogue(dialogue) => self.validate_dialogue(dialogue),
            Line::Branches(cond) => self.validate_branches(cond),
            Line::Choices(choices) => self.validate_choices(choices),
            Line::Goto(goto) => self.validate_goto(&goto.goto),
            Line::SetCmd(cmd) => self.validate_state(&cmd.set),
            Line::Cmds(cmds) => self.validate_cmds(&cmds),
            _ => Ok(()),
        }
    }

    /// Validates that a line (either text or dialogue) has valid HTML and valid speakers.
    fn validate_passage(&self, lines: &Passage) -> Result<(), Error> {
        for (i, line) in lines.iter().enumerate() {
            if let Err(e) = self.validate_line(line) {
                return Err(error!("Line {}: {}", i + 1, e));
            }
        }
        Ok(())
    }

    /// Validates an operator on a given value.
    /// Any value supports assignment, but only Numbers can be added or subtracted.
    fn validate_op(v1: &Value, v2: &Value, op: Operator) -> Result<(), Error> {
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
    fn validate_cmp(v1: &Value, v2: &Value, cmp: Comparator) -> Result<(), Error> {
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
    fn validate_var(&self, var: &str) -> Result<&Value, Error> {
        match self.story.value(&QualifiedName::from(self.namespace, var)) {
            Some(value) => Ok(value),
            None => return Err(error!("No state variable named '{}'", var)),
        }
    }

    /// Validates the state only contains configured keys.
    fn validate_state(&self, state: &State) -> Result<(), Error> {
        for (key, value) in state {
            let smod = StateMod::from_str(key)?;
            let state_value = self.validate_var(smod.var)?;
            Self::validate_op(state_value, value, smod.op)?;
        }
        Ok(())
    }

    fn validate_goto(&self, passage_name: &str) -> Result<(), Error> {
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
    fn validate_choices(&self, choices: &Choices) -> Result<(), Error> {
        for (key, choice) in &choices.choices {
            match choice {
                Choice::PassageName(passage_name) => self.validate_goto(passage_name)?,
                Choice::Conditional(conditional) => {
                    for (_choice_name, passage_name) in conditional {
                        self.validate_conditional(key)?;
                        self.validate_goto(passage_name)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_passages(&self, passages: &Passages) -> Result<(), Error> {
        for (passage_name, passage) in passages {
            if let Err(e) = self.validate_passage(passage) {
                return Err(error!("Passage '{}': {}", passage_name, e));
            }
        }
        Ok(())
    }

    /// Validates an entire story for valid passage references, HTML, conditionals.
    pub fn validate(&mut self) -> Result<(), Error> {
        for (namespace, namespace_val) in self.story {
            self.namespace = namespace;
            self.validate_passages(&namespace_val.passages)?;
        }
        Ok(())
    }
}
