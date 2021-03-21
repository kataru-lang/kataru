use crate::{
    error::{Error, Result},
    structs::{
        Bookmark, Branchable, Choices, Dialogue, Line, Passage, QualifiedName, Return, State,
        StateUpdatable, Story, GLOBAL,
    },
    traits::CopyMerge,
    vars::replace_var,
    Cmd, Params, StoryGetters, Value,
};

static RETURN: Line = Line::Return(Return { r#return: () });
static EMPTY_PASSAGE: Passage = Vec::new();

pub struct Runner<'r> {
    pub bookmark: &'r mut Bookmark,
    pub story: &'r Story,
    pub line_num: usize,
    pub passage: &'r Passage,
    lines: Vec<&'r Line>,
    line: Line,
    choices: Choices,
    breaks: Vec<usize>,
    speaker: String,
}

impl<'r> Runner<'r> {
    pub fn new(bookmark: &'r mut Bookmark, story: &'r Story) -> Result<Self> {
        // Flatten dialogue lines
        let mut runner = Self {
            bookmark,
            story,
            line_num: 0,
            lines: Vec::new(),
            line: Line::Continue,
            passage: &EMPTY_PASSAGE,
            choices: Choices::default(),
            breaks: Vec::new(),
            speaker: "".to_string(),
        };
        runner.goto()?;
        Ok(runner)
    }

    /// Gets the next dialogue line from the story based on the user's input.
    /// Internally, a single call to `next()` may result in multiple lines being processed,
    /// i.e. when a choice is being made.
    pub fn next(&mut self, input: &str) -> Result<&Line> {
        self.line = self.process(input)?;
        while self.line == Line::Continue {
            self.line = self.process("")?;
        }
        Ok(&self.line)
    }

    /// Call the configured passage by putting return position on stack.
    /// And goto the passage.
    pub fn call(&mut self, passage: String) -> Result<()> {
        self.bookmark.position.line += 1;

        // Don't push this func onto the stack of the next line is just a return.
        if let Line::Return(_) = self.lines[self.bookmark.position.line] {
        } else {
            self.bookmark.stack.push(self.bookmark.position.clone());
        }

        self.bookmark.position.passage = passage;
        self.bookmark.position.line = 0;
        self.goto()?;
        Ok(())
    }

    /// Go to the passage specified in bookmark.
    /// This public API method automatically triggers `run_on_passage`.
    pub fn goto(&mut self) -> Result<()> {
        self.load_bookmark_position()?;
        self.run_on_passage()?;
        Ok(())
    }

    pub fn save_snapshot(&mut self, name: &str) {
        self.bookmark.save_snapshot(name)
    }

    pub fn load_snapshot(&mut self, name: &str) -> Result<()> {
        self.bookmark.load_snapshot(name)?;
        self.load_bookmark_position()?;

        // Preload choices if loading a snapshot paused on choices.
        if let Line::RawChoices(choices) = self.lines[self.bookmark.position.line] {
            self.line = Line::Choices(Choices::get_valid(choices, &self.bookmark)?)
        }
        Ok(())
    }

    /// Loads lines into a single flat array of references.
    /// Initializes breakpoint stack.
    fn load_passage(&mut self, lines: &'r [Line]) {
        self.lines = vec![];
        self.load_lines(lines);
        self.lines.push(&RETURN);

        self.breaks = vec![];
        self.load_breaks();
    }

    /// Initialize the line break stack.
    /// Loop through each line in the flattened array until current line
    /// number is reached.
    /// Each time a branch is detected, push the end of the branch on the break stack.
    fn load_breaks(&mut self) {
        for (line_num, line) in self.lines.iter().enumerate() {
            if line_num >= self.bookmark.position.line {
                break;
            }
            match line {
                Line::Break => {
                    self.breaks.pop();
                }
                Line::Branches(branches) => {
                    self.breaks.push(line_num + branches.len());
                }
                _ => (),
            }
        }
    }

    /// Loads lines into a single flat array of references.
    fn load_lines(&mut self, lines: &'r [Line]) {
        for line in lines {
            match line {
                Line::Branches(branches) => {
                    self.lines.push(&line);

                    // Add breaks after each line except for the last line
                    let mut branches_it = branches.iter();
                    if let Some((_expression, branch_lines)) = branches_it.next() {
                        self.load_lines(branch_lines);
                    }
                    for (_expression, branch_lines) in branches_it {
                        self.lines.push(&Line::Break);
                        self.load_lines(branch_lines);
                    }
                }
                _ => self.lines.push(&line),
            }
        }
    }

    /// Attempts to get a passage matching `qname`.
    /// First checks in the specified namespace, and falls back to root namespace if not found.
    ///
    /// Note that passage name could be:
    /// 1. a local name (unquallified), in which case namespace stays the same.
    /// 2. a qualified name pointing to another section, in which case we switch namespace.
    /// 3. a global name, in which we must changed namespace to root.
    fn get_passage(&mut self, qname: QualifiedName) -> Result<&'r Passage> {
        // First try to find the section specified namespace.
        if let Some(section) = self.story.get(&qname.namespace) {
            if let Some(passage) = section.passage(&qname.name) {
                // Case 2: name is not local, so switch namespace.
                self.bookmark.position.namespace = qname.namespace;
                self.bookmark.position.passage = qname.name;
                return Ok(passage);
            }
        } else {
            return Err(error!("Invalid namespace '{}'", &qname.namespace));
        }

        // Fall back to try global namespace.
        if let Some(global_section) = self.story.get(GLOBAL) {
            if let Some(passage) = global_section.passage(&qname.name) {
                // Case 3: passage could not be found in local/specified namespace, so switch to global.
                self.bookmark.position.namespace = GLOBAL.to_string();
                self.bookmark.position.passage = qname.name;
                return Ok(passage);
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

    /// Runs the `onPassage` set command.
    fn run_on_passage(&mut self) -> Result<()> {
        let namespace = &self.bookmark.position.namespace;

        // First try to find the section specified namespace.
        if let Some(section) = self.story.get(namespace) {
            if let Some(set_cmd) = &section.config.on_passage {
                return self.set_state(&set_cmd.set);
            }
            Ok(())
        } else {
            Err(error!("Invalid namespace '{}'", &namespace))
        }
    }

    /// Gets the current passage based on the bookmark's position.
    /// Loads the lines into its flattened form.
    /// Automatically handles updating of namespace.
    fn load_bookmark_position(&mut self) -> Result<()> {
        let qname = QualifiedName::from(
            &self.bookmark.position.namespace,
            &self.bookmark.position.passage,
        );
        self.passage = self.get_passage(qname)?;
        self.load_passage(self.passage);
        Ok(())
    }

    fn set_state(&mut self, state: &State) -> Result<()> {
        let passage_name = self.bookmark.position.passage.clone();
        let root_sets = self.bookmark.state()?.update(&state, &passage_name)?;
        self.bookmark
            .root_state()?
            .update(&root_sets, &passage_name)?;
        Ok(())
    }

    fn get_default_params(&self, command_name: &str) -> Option<&Params> {
        Some(
            self.story
                .params(&QualifiedName::from(
                    &self.bookmark.position.namespace,
                    command_name,
                ))?
                .as_ref()?,
        )
    }

    /// If `character` is local, then prepend the namespace to the character.command.
    fn get_qualified_command(&self, character: &str, command_name: &str) -> String {
        // If currently in global namespace, don't bother checking.
        if self.bookmark.position.namespace == GLOBAL {
            return format!("{}.{}", character, command_name);
        }

        // If this is a local character, then prepend the namespace to the command.
        if let Some(section) = self.story.get(&self.bookmark.position.namespace) {
            section.config.characters.contains_key(character);
            format!(
                "{}:{}.{}",
                &self.bookmark.position.namespace, character, command_name
            )
        } else {
            format!("{}.{}", character, command_name)
        }
    }

    /// Get the vector of qualified commands with default parameters included.
    fn get_full_commands(&self, commands: &Vec<Cmd>) -> Result<Vec<Cmd>> {
        let mut full_commands: Vec<Cmd> = Vec::new();
        for command in commands {
            for (command_name, params) in command {
                let mut cmd = Cmd::new();
                let mut merged_params = Params::new();

                let split: Vec<&str> = command_name.split(".").collect();

                // Handle character commands
                let normalized_name: String;
                let qualified_command: String;
                match split.as_slice() {
                    [character, command_name] => {
                        normalized_name = format!("$character.{}", command_name);
                        qualified_command = self.get_qualified_command(character, command_name);
                    }
                    [command_name] => {
                        normalized_name = command_name.to_string();
                        qualified_command = command_name.to_string();
                    }
                    _ => return Err(error!("Commands can only contain one '.' delimeter.")),
                };

                if let Some(default_params) = self.get_default_params(&normalized_name) {
                    merged_params = params.copy_merge(default_params)?;

                    // If the params have variable names, replace with variable value.
                    for (_var, val) in merged_params.iter_mut() {
                        if let Value::String(text) = val {
                            if let Some(replaced) = replace_var(text, &self.bookmark)? {
                                *val = replaced;
                            }
                        }
                    }
                }

                cmd.insert(qualified_command, merged_params);
                full_commands.push(cmd);
            }
        }
        Ok(full_commands)
    }

    /// Processes a line.
    /// Returning Line::Continue signals to `next()` that another line should be processed
    /// before returning a line to the user.
    fn process_line(&mut self, input: &str, line: &'r Line) -> Result<Line> {
        let line = match &line {
            // When a choice is encountered, it should first be returned for display.
            // Second time it's encountered, go to the chosen passage.
            Line::RawChoices(choices) => {
                // If empty input, choices are being returned for display.
                if input.is_empty() {
                    Line::Choices(Choices::get_valid(choices, &self.bookmark)?)
                } else {
                    if let Line::Choices(ref mut choices) = self.line {
                        // If first attempt, use choices saved in self.line.
                        if let Some(passage_name) = choices.choices.remove(input) {
                            self.call(passage_name)?;
                            Line::Continue
                        } else {
                            // Move all choices out of their map saved in self.line.
                            self.choices = Choices::from(choices)?;
                            Line::InvalidChoice
                        }
                    // For second attempt, use self.choices.
                    } else if let Some(passage_name) = self.choices.choices.remove(input) {
                        self.call(passage_name)?;
                        Line::Continue
                    } else {
                        Line::InvalidChoice
                    }
                }
            }
            Line::Choices(_) => {
                return Err(error!("Mutated choices were found."));
            }
            // When input is encountered, it should first be returned for display.
            // Second time it's encountered, modify state.
            Line::Input(input_cmd) => {
                if input.is_empty() {
                    line.clone()
                } else {
                    for (var, _prompt) in &input_cmd.input {
                        let mut state = State::new();
                        state.insert(var.clone(), Value::String(input.to_string()));
                        self.set_state(&state)?
                    }
                    self.bookmark.position.line += 1;
                    Line::Continue
                }
            }
            Line::Branches(branches) => {
                let skipped_len = branches.take(&mut self.bookmark)?;
                let branch_len = branches.length();
                self.breaks
                    .push(self.bookmark.position.line + branch_len - skipped_len);
                Line::Continue
            }
            Line::Call(call) => {
                self.call(call.passage.clone())?;
                Line::Continue
            }
            Line::Return(_) => match self.bookmark.stack.pop() {
                Some(position) => {
                    self.bookmark.position = position;
                    self.load_bookmark_position()?;
                    Line::Continue
                }
                None => Line::End,
            },
            Line::Break => {
                let last_break = self.breaks.pop();
                self.bookmark.position.line = match last_break {
                    Some(line_num) => line_num,
                    None => 0,
                };
                Line::Continue
            }
            Line::Commands(commands) => {
                self.bookmark.position.line += 1;
                let full_commands = self.get_full_commands(commands)?;
                Line::Commands(full_commands)
            }
            Line::SetCmd(set) => {
                let passage_name = self.bookmark.position.passage.clone();
                let root_sets = self.bookmark.state()?.update(&set.set, &passage_name)?;
                self.bookmark
                    .root_state()?
                    .update(&root_sets, &passage_name)?;
                self.bookmark.position.line += 1;
                Line::Continue
            }
            Line::RawDialogue(map) => {
                self.bookmark.position.line += 1;
                let dialogue = Dialogue::from_map(map, &self.story, &self.bookmark)?;
                self.speaker = dialogue.name.clone();
                Line::Dialogue(dialogue)
            }
            Line::Dialogue(dialogue) => {
                self.bookmark.position.line += 1;
                Line::Dialogue(dialogue.clone())
            }
            Line::Text(text) => {
                self.bookmark.position.line += 1;
                Line::Dialogue(Dialogue::from(
                    &self.speaker,
                    text,
                    self.story,
                    self.bookmark,
                )?)
            }
            Line::Continue => {
                self.bookmark.position.line += 1;
                Line::Continue
            }
            Line::End => Line::End,
            Line::InvalidChoice => Line::InvalidChoice,
        };
        Ok(line)
    }

    /// If the current configuration points to a valid line, processes the line.
    fn process(&mut self, input: &str) -> Result<Line> {
        if self.bookmark.position.line >= self.lines.len() {
            Err(error!(
                "Invalid line number {} in passage {}",
                self.bookmark.position.line, self.bookmark.position.passage
            ))
        } else {
            self.process_line(input, self.lines[self.bookmark.position.line])
        }
    }
}
