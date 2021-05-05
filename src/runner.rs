use crate::{
    error::{Error, Result},
    structs::{
        Bookmark, Branchable, Choices, CommandGetters, Dialogue, Line, Passage, QualifiedName,
        Return, State, Story, StoryGetters,
    },
    Section, Value,
};

static RETURN: Line = Line::Return(Return { r#return: () });
static EMPTY_PASSAGE: Passage = Vec::new();
lazy_static! {
    static ref EMPTY_SECTION: Section = Section::default();
}

pub struct Runner<'r> {
    pub bookmark: &'r mut Bookmark,
    pub story: &'r Story,
    pub line_num: usize,
    pub passage: &'r Passage,
    pub section: &'r Section,
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
            section: &EMPTY_SECTION,
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

    /// Returns true if tail call optimization is possible.
    /// This requires that the current line is a return statement, and
    /// that this section has no `on_exit` callback.
    fn can_optimize_tail_call(&self) -> bool {
        if let Line::Return(_) = self.lines[self.bookmark.line()] {
            return self.section.on_exit().is_none();
        }
        false
    }

    /// Call the configured passage by putting return position on stack.
    /// And goto the passage.
    pub fn call(&mut self, passage: String) -> Result<()> {
        self.bookmark.next_line();

        // Don't push this func onto the stack of the next line is just a return.
        // (Tail call optimization).
        if !self.can_optimize_tail_call() {
            self.bookmark.stack.push(self.bookmark.position().clone());
        }

        self.bookmark.set_passage(passage);
        self.bookmark.set_line(0);
        self.goto()?;
        Ok(())
    }

    /// Go to the passage specified in bookmark.
    /// This public API method automatically triggers `run_on_passage`.
    pub fn goto(&mut self) -> Result<()> {
        self.load_bookmark_position()?;
        self.run_on_enter()?;
        Ok(())
    }

    pub fn save_snapshot(&mut self, name: &str) {
        self.bookmark.save_snapshot(name)
    }

    pub fn load_snapshot(&mut self, name: &str) -> Result<()> {
        self.bookmark.load_snapshot(name)?;
        self.load_bookmark_position()?;

        // Preload choices if loading a snapshot paused on choices.
        if let Line::RawChoices(choices) = self.lines[self.bookmark.line()] {
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
    /// We must remove breaks that we pass through.
    fn load_breaks(&mut self) {
        for (line_num, line) in self.lines.iter().enumerate() {
            if line_num >= self.bookmark.line() {
                break;
            }

            // If we pass the last break, remove it from the stack.
            if let Some(last_break) = self.breaks.last() {
                if line_num > *last_break {
                    self.breaks.pop();
                }
            }
            match line {
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
                    let mut branches_it = branches.exprs.iter();
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

    /// Runs the `onEnter` set command.
    fn run_on_enter(&mut self) -> Result<()> {
        if let Some(set_cmd) = &self.section.on_enter() {
            return self.bookmark.set_state(&set_cmd.set);
        }
        Ok(())
    }

    /// Runs the `onEnter` set command.
    fn run_on_exit(&mut self) -> Result<()> {
        if let Some(set_cmd) = &self.section.on_exit() {
            return self.bookmark.set_state(&set_cmd.set);
        }
        Ok(())
    }

    /// Gets the current passage based on the bookmark's position.
    /// Loads the lines into its flattened form.
    /// Automatically handles updating of namespace.
    fn load_bookmark_position(&mut self) -> Result<()> {
        let mut qname = QualifiedName::from(self.bookmark.namespace(), self.bookmark.passage());
        let (section, passage) = self.story.section_for_passage(&mut qname)?;
        self.section = section;
        self.passage = passage;
        self.bookmark.update_position(qname);

        self.load_passage(self.passage);
        Ok(())
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
                        self.bookmark.set_state(&state)?
                    }
                    self.bookmark.next_line();
                    Line::Continue
                }
            }
            Line::Branches(branches) => {
                let skipped_len = branches.take(&mut self.bookmark)?;
                let branch_len = branches.len();
                self.breaks
                    .push(self.bookmark.line() + branch_len - skipped_len);
                Line::Continue
            }
            Line::Call(call) => {
                self.call(call.passage.clone())?;
                Line::Continue
            }
            Line::Return(_) => {
                self.run_on_exit()?;
                match self.bookmark.stack.pop() {
                    Some(position) => {
                        self.bookmark.set_position(position);
                        self.load_bookmark_position()?;
                        Line::Continue
                    }
                    None => Line::End,
                }
            }
            Line::Break => {
                let last_break = self.breaks.pop();
                self.bookmark.set_line(match last_break {
                    Some(line_num) => line_num,
                    None => 0,
                });
                Line::Continue
            }
            Line::Command(command) => {
                self.bookmark.next_line();
                let full_command = command.get_full_command(&self.story, &self.bookmark)?;
                Line::Command(full_command)
            }
            Line::PositionalCommand(positional_command) => {
                self.bookmark.next_line();
                let full_command =
                    positional_command.get_full_command(&self.story, &self.bookmark)?;
                Line::Command(full_command)
            }
            Line::SetCommand(set) => {
                self.bookmark.next_line();
                self.bookmark.set_state(&set.set)?;
                Line::Continue
            }
            Line::RawDialogue(map) => {
                self.bookmark.next_line();
                let dialogue = Dialogue::from_map(map, &self.story, &self.bookmark)?;
                self.speaker = dialogue.name.clone();
                Line::Dialogue(dialogue)
            }
            Line::Dialogue(dialogue) => {
                self.bookmark.next_line();
                Line::Dialogue(dialogue.clone())
            }
            Line::Text(text) => {
                self.bookmark.next_line();
                Line::Dialogue(Dialogue::from(
                    &self.speaker,
                    text,
                    self.story,
                    self.bookmark,
                )?)
            }
            Line::Continue => {
                self.bookmark.next_line();
                Line::Continue
            }
            Line::End => Line::End,
            Line::InvalidChoice => Line::InvalidChoice,
        };
        Ok(line)
    }

    /// If the current configuration points to a valid line, processes the line.
    fn process(&mut self, input: &str) -> Result<Line> {
        if self.bookmark.line() >= self.lines.len() {
            Err(error!(
                "Invalid line number {} in passage '{}'",
                self.bookmark.line(),
                self.bookmark.passage()
            ))
        } else {
            self.process_line(input, self.lines[self.bookmark.line()])
        }
    }
}
