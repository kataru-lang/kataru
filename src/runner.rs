use crate::{
    error::{Error, Result},
    structs::{
        Bookmark, Branchable, Choices, CommandGetters, Dialogue, Passage, QualifiedName,
        RawChoices, RawLine, Return, State, Story, StoryGetters,
    },
    Line, Map, Section, Value,
};

static RETURN: RawLine = RawLine::Return(Return { r#return: () });
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
    lines: Vec<&'r RawLine>,
    choice_to_passage: Map<&'r str, &'r str>,
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
            passage: &EMPTY_PASSAGE,
            section: &EMPTY_SECTION,
            choice_to_passage: Map::new(),
            breaks: Vec::new(),
            speaker: "".to_string(),
        };
        runner.goto()?;
        Ok(runner)
    }

    fn readline(&self) -> Result<&'r RawLine> {
        if self.bookmark.line() >= self.lines.len() {
            return Err(error!(
                "Invalid line number {} in passage '{}'",
                self.bookmark.line(),
                self.bookmark.passage()
            ));
        };
        Ok(self.lines[self.bookmark.line()])
    }

    /// Gets the next dialogue line from the story based on the user's input.
    /// Internally, a single call to `next()` may result in multiple lines being processed,
    /// i.e. when a choice is being made.
    pub fn next(&mut self, mut input: &str) -> Result<Line> {
        loop {
            let raw_line = self.readline()?;
            match raw_line {
                // When a choice is encountered, it should first be returned for display.
                // Second time it's encountered, go to the chosen passage.
                RawLine::Choices(raw_choices) => {
                    // If empty input, choices are being returned for display.
                    if input.is_empty() {
                        let choices = self.load_choices(raw_choices)?;
                        // If no options and a default was provided, call the default.
                        if choices.choices.is_empty() && !choices.default.is_empty() {
                            self.call(choices.default.clone())?;
                        } else {
                            return Ok(Line::Choices(choices));
                        }
                    } else {
                        if let Some(passage_name) = self.choice_to_passage.remove(input) {
                            self.call(passage_name.to_string())?;
                        } else {
                            return Ok(Line::InvalidChoice);
                        }
                    }
                }
                // When input is encountered, it should first be returned for display.
                // Second time it's encountered, modify state.
                RawLine::Input(input_cmd) => {
                    if input.is_empty() {
                        return Ok(Line::Input(input_cmd.clone()));
                    } else {
                        for (var, _prompt) in &input_cmd.input {
                            let mut state = State::new();
                            state.insert(var.clone(), Value::String(input.to_string()));
                            self.bookmark.set_state(&state)?
                        }
                        self.bookmark.next_line();
                    }
                }
                RawLine::Branches(branches) => {
                    let skipped_len = branches.take(&mut self.bookmark)?;
                    let branch_len = branches.len();
                    self.breaks
                        .push(self.bookmark.line() + branch_len - skipped_len);
                }
                RawLine::Call(call) => {
                    self.call(call.passage.clone())?;
                }
                RawLine::Return(_) => {
                    self.run_on_exit()?;
                    match self.bookmark.stack.pop() {
                        Some(position) => {
                            self.bookmark.set_position(position);
                            self.load_bookmark_position()?;
                        }
                        None => return Ok(Line::End),
                    }
                }
                RawLine::Break => {
                    let last_break = self.breaks.pop();
                    self.bookmark.set_line(match last_break {
                        Some(line_num) => line_num,
                        None => 0,
                    });
                }
                RawLine::Command(raw_command) => {
                    self.bookmark.next_line();
                    let command = raw_command.get_full_command(&self.story, &self.bookmark)?;
                    return Ok(Line::Command(command));
                }
                RawLine::PositionalCommand(positional_command) => {
                    self.bookmark.next_line();
                    let command =
                        positional_command.get_full_command(&self.story, &self.bookmark)?;
                    return Ok(Line::Command(command));
                }
                RawLine::SetCommand(set) => {
                    self.bookmark.next_line();
                    self.bookmark.set_state(&set.set)?;
                }
                RawLine::Dialogue(map) => {
                    self.bookmark.next_line();
                    let dialogue = Dialogue::from_map(map, &self.story, &self.bookmark)?;
                    self.speaker = dialogue.name.clone();
                    return Ok(Line::Dialogue(dialogue));
                }
                RawLine::Text(text) => {
                    self.bookmark.next_line();
                    return Ok(Line::Dialogue(Dialogue::from(
                        &self.speaker,
                        text,
                        self.story,
                        self.bookmark,
                    )?));
                }
                _ => return Err(error!("Unknown error.")),
            };
            input = "";
        }
    }

    /// Returns true if tail call optimization is possible.
    /// This requires that the current line is a return statement, and
    /// that this section has no `on_exit` callback.
    fn can_optimize_tail_call(&self) -> bool {
        if let RawLine::Return(_) = self.lines[self.bookmark.line()] {
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

    /// Repopulates `self` with a list of all valid choices from `raw` in order.
    /// Also repopulates the `choice_to_passage` map which updated mappings.
    pub fn load_choices(&mut self, raw: &'r RawChoices) -> Result<Choices> {
        let choices = Choices::from_raw(&mut self.choice_to_passage, raw, &self.bookmark)?;
        Ok(choices)
    }

    pub fn load_snapshot(&mut self, name: &str) -> Result<()> {
        self.bookmark.load_snapshot(name)?;
        self.load_bookmark_position()?;
        if let RawLine::Choices(raw_choices) = self.readline()? {
            self.load_choices(raw_choices)?;
        }

        Ok(())
    }

    /// Loads lines into a single flat array of references.
    /// Initializes breakpoint stack.
    fn load_passage(&mut self, lines: &'r [RawLine]) {
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
                RawLine::Branches(branches) => {
                    self.breaks.push(line_num + branches.len());
                }
                _ => (),
            }
        }
    }

    /// Loads lines into a single flat array of references.
    fn load_lines(&mut self, lines: &'r [RawLine]) {
        for line in lines {
            match line {
                RawLine::Branches(branches) => {
                    self.lines.push(&line);

                    // Add breaks before each lines except the first.
                    let mut is_first = true;
                    for (_expression, branch_lines) in &branches.exprs {
                        if !is_first {
                            self.lines.push(&RawLine::Break);
                        }
                        self.load_lines(branch_lines);
                        is_first = false;
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
}
