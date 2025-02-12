/// Public `Runner` interface for Kataru.
use crate::{
    error::{Error, Result},
    structs::{
        Bookmark, Branches, Call, ChoiceTarget, Choices, CommandGetters, Dialogue,
        PositionalCommand, QualifiedName, RawChoice, RawChoices, RawCommand, RawLine, Section,
        State, Story,
    },
    Input, Line, Map, Save, SetCommand, StateMod, Validator, Value,
};

lazy_static! {
    static ref EMPTY_SECTION: Section = Section::default();
    static ref EMPTY_STORY: Story = Story::default();
}

/// Public struct for running Kataru scripts.
#[self_referencing]
pub struct Runner {
    /// Story to read. Cannot be moved or will cause compiler error, since state holds references to it.
    story: Story,

    /// Internal state.
    #[borrows(story)]
    #[covariant]
    state: RunnerState<'this>,
}

/// Public interface to runner.
impl Runner {
    /// Construct a dialogue runner from a bookmark and a story.
    /// Both are moved into the runner. Story is moved to a pinned location
    /// on the heap so that it can't be moved.
    pub fn init(bookmark: Bookmark, story: Story, validate: bool) -> Result<Self> {
        let mut runner: Self = RunnerTryBuilder {
            story,
            state_builder: |story| RunnerState::try_new(bookmark, story),
        }
        .try_build()?;
        if validate {
            runner.validate()?;
        }
        Ok(runner)
    }

    /// Load a bookmark.
    pub fn load_bookmark(&mut self, bookmark: Bookmark) -> Result<()> {
        self.with_state_mut(|state| state.load_bookmark(bookmark))
    }

    /// Go to the passage specified in bookmark.
    /// This public API method automatically triggers `run_on_passage`.
    pub fn goto(&mut self, passage_name: String) -> Result<()> {
        self.with_state_mut(|state| state.goto(passage_name))
    }

    /// Set the bookmark and goto that passage. Run the first line and return.
    /// This clears the stack.
    pub fn run(&mut self, passage_name: String) -> Result<Line> {
        self.with_state_mut(|state| state.run(passage_name))
    }

    /// Validate the story.
    pub fn validate(&mut self) -> Result<()> {
        self.with_state_mut(|state| state.validate())
    }

    /// Save a snapshot of the current position to be loaded later.
    pub fn save_snapshot(&mut self, name: &str) {
        self.with_state_mut(|state| state.save_snapshot(name))
    }

    // Load a previously named snapshot.
    pub fn load_snapshot(&mut self, name: &str) -> Result<()> {
        self.with_state_mut(|state| state.load_snapshot(name))
    }

    /// Save the story to the given path.
    pub fn load_story(&mut self, story: Story, validate: bool) -> Result<()> {
        *self = Runner::init(self.bookmark().clone(), story, validate)?;
        Ok(())
    }

    /// Save the story to the given path.
    pub fn save_story(&self, path: &str) -> Result<()> {
        self.story().save(path)
    }

    /// Save the bookmark to the given path.
    pub fn save_bookmark(&self, path: &str) -> Result<()> {
        self.bookmark().save(path)
    }

    /// Gets the next dialogue line from the story based on the user's input.
    /// Internally, a single call to `next()` may result in multiple lines being processed,
    /// i.e. when a choice is being made.
    pub fn next(&mut self, input: &str) -> Result<Line> {
        self.with_state_mut(|state| state.next(input))
    }

    /// Public getter for the story.
    pub fn story(&self) -> &Story {
        self.borrow_story()
    }

    /// Public getter for the bookmark.
    pub fn bookmark(&self) -> &Bookmark {
        &self.borrow_state().bookmark
    }

    /// Public getter for the current namespace.
    pub fn namespace(&self) -> &str {
        self.borrow_state().bookmark.namespace()
    }

    /// Public getter for the current passage.
    pub fn passage(&self) -> &str {
        self.borrow_state().bookmark.passage()
    }

    /// Public setter for setting values.
    pub fn set_state(&mut self, statemod: StateMod, value: Value) -> Result<()> {
        self.with_state_mut(|state| state.set_state(statemod, value))
    }

    /// Public getter for variable state.
    pub fn get_state(&self, varname: &str) -> Result<&Value> {
        self.with_state(|state| state.get_state(varname))
    }

    /// Sets the line number in the bookmark.
    pub fn set_line(&mut self, line_num: usize) {
        self.with_state_mut(|state| state.bookmark.set_line(line_num));
    }

    /// Gets the line number in the bookmark.
    pub fn line(&self) -> usize {
        self.with_state(|state| state.bookmark.line())
    }

    /// Clears the stack.
    pub fn clear_stack(&mut self) {
        self.with_state_mut(|state| state.bookmark.stack.clear());
    }
}

/// Internal struct used for the flattened array of lines.
/// Each element is either a raw line reference,
/// or a break statement pointing to the line to jump to.
#[derive(Debug, Clone, Copy)]
enum LineRef<'story> {
    Branches(&'story Branches),
    SetCommand(&'story SetCommand),
    Input(&'story Input),
    Choices(&'story RawChoices),
    Command(&'story RawCommand),
    PositionalCommand(&'story PositionalCommand),
    Call(&'story Call),
    Return,
    Text(&'story String),
    Dialogue(&'story Map<String, String>),
    Break(usize),
}
impl<'story> From<&'story RawLine> for LineRef<'story> {
    fn from(raw: &'story RawLine) -> Self {
        match raw {
            RawLine::Branches(line_ref) => Self::Branches(line_ref),
            RawLine::SetCommand(line_ref) => Self::SetCommand(line_ref),
            RawLine::Input(line_ref) => Self::Input(line_ref),
            RawLine::Choices(line_ref) => Self::Choices(line_ref),
            RawLine::Command(line_ref) => Self::Command(line_ref),
            RawLine::PositionalCommand(line_ref) => Self::PositionalCommand(line_ref),
            RawLine::Call(line_ref) => Self::Call(line_ref),
            RawLine::Return(_) => Self::Return,
            RawLine::Text(line_ref) => Self::Text(line_ref),
            RawLine::Dialogue(line_ref) => Self::Dialogue(line_ref),
        }
    }
}

/// Internal runner state.
struct RunnerState<'story> {
    /// Reference to bookmark to mutate as we progress through the story.
    bookmark: Bookmark,
    /// The story the runner references.
    story: &'story Story,
    /// Flattened array of line references (use `line_num` to index).
    lines: Vec<LineRef<'story>>,
    /// Loaded choice-to-passage mapping from last choices seen.
    choice_to_passage: Map<&'story str, &'story str>,
    /// Loaded choice-to-line-num mapping from last choices seen.
    choice_to_line_num: Map<&'story str, usize>,
    /// Last known speaker.
    speaker: String,
}

impl<'story> RunnerState<'story> {
    pub fn try_new(bookmark: Bookmark, story: &'story Story) -> Result<Self> {
        let mut state = Self {
            bookmark,
            story,
            lines: Vec::default(),
            choice_to_passage: Map::default(),
            choice_to_line_num: Map::default(),
            speaker: String::default(),
        };
        state.bookmark.init_state(state.story);
        if !state.bookmark.passage().is_empty() {
            state.load_passage()?;
        }
        Ok(state)
    }

    /// Load a new bookmark.
    pub fn load_bookmark(&mut self, bookmark: Bookmark) -> Result<()> {
        self.bookmark = bookmark;
        self.bookmark.init_state(self.story);
        self.load_passage()
    }

    /// Go to the passage specified in bookmark.
    /// This public API method automatically triggers `run_on_passage`.
    pub fn goto(&mut self, passage_name: String) -> Result<()> {
        self.bookmark.set_passage(passage_name);
        self.bookmark.set_line(0);
        self.load_passage()?;
        self.run_on_enter()?;
        Ok(())
    }

    /// Set the bookmark and goto that passage. Run the first line and return.
    /// This clears the stack.
    pub fn run(&mut self, passage_name: String) -> Result<Line> {
        self.goto(passage_name)?;
        self.bookmark.stack.clear();
        self.next("")
    }

    /// Validate the story.
    pub fn validate(&mut self) -> Result<()> {
        Validator::new(self.story, &mut self.bookmark).validate()
    }

    /// Save a snapshot of the current position to be loaded later.
    pub fn save_snapshot(&mut self, name: &str) {
        self.bookmark.save_snapshot(name)
    }

    // Load a previously named snapshot.
    pub fn load_snapshot(&mut self, name: &str) -> Result<()> {
        self.bookmark.load_snapshot(name)?;
        self.load_passage()?;
        if let LineRef::Choices(raw_choices) = self.readline()? {
            self.load_choices(raw_choices)?;
        }

        Ok(())
    }

    /// Gets the next dialogue line from the story based on the user's input.
    /// Internally, a single call to `next()` may result in multiple lines being processed,
    /// i.e. when a choice is being made.
    pub fn next(&mut self, mut input: &str) -> Result<Line> {
        loop {
            let line_ref = self.readline()?;
            // println!("Run L{}: {:#?}", self.bookmark.position().line, line_ref);
            match line_ref {
                // When a choice is encountered, it should first be returned for display.
                // Second time it's encountered, go to the chosen passage.
                LineRef::Choices(raw_choices) => {
                    // If empty input, choices are being returned for display.
                    if input.is_empty() {
                        let choices = self.load_choices(raw_choices)?;
                        // If no choices, call the default.
                        if choices.is_empty() {
                            self.call_default(raw_choices)?
                        } else {
                            return Ok(Line::Choices(choices));
                        }
                    } else {
                        // If should jump to passage.
                        if let Some(passage_name) = self.choice_to_passage.remove(input) {
                            self.call_choice(raw_choices, passage_name.to_string())?;
                        }
                        // If should jump to line number.
                        else if let Some(skip_lines) = self.choice_to_line_num.remove(input) {
                            self.bookmark.skip_lines(skip_lines + 1);
                        } else {
                            return Ok(Line::InvalidChoice);
                        }
                    }
                }
                // When input is encountered, it should first be returned for display.
                // Second time it's encountered, modify state.
                LineRef::Input(input_cmd) => {
                    if input.is_empty() {
                        return Ok(Line::Input(input_cmd.clone()));
                    } else {
                        for var in input_cmd.input.keys() {
                            let mut state = State::new();
                            state.insert(var.clone(), Value::String(input.to_string()));
                            self.bookmark.set_state(&state)?
                        }
                        self.bookmark.next_line();
                    }
                }
                LineRef::Branches(branches) => {
                    branches.take(&mut self.bookmark)?;
                }
                LineRef::Call(call) => {
                    self.call(call.passage.clone())?;
                }
                LineRef::Return => {
                    self.run_on_exit()?;
                    match self.bookmark.stack.pop() {
                        Some(position) => {
                            self.bookmark.set_position(position);
                            self.load_passage()?;
                        }
                        None => return Ok(Line::End),
                    }
                }
                LineRef::Break(line_num) => self.bookmark.set_line(line_num),
                LineRef::Command(raw_command) => {
                    self.bookmark.next_line();
                    let command = raw_command.build_command(self.story, &self.bookmark)?;
                    return Ok(Line::Command(command));
                }
                LineRef::PositionalCommand(positional_command) => {
                    self.bookmark.next_line();
                    let command = positional_command.build_command(self.story, &self.bookmark)?;
                    return Ok(Line::Command(command));
                }
                LineRef::SetCommand(set) => {
                    self.bookmark.next_line();
                    self.bookmark.set_state(&set.set)?;
                }
                LineRef::Dialogue(map) => {
                    self.bookmark.next_line();
                    let dialogue = Dialogue::from_map(map, self.story, &self.bookmark)?;
                    self.speaker = dialogue.name.clone();
                    return Ok(Line::Dialogue(dialogue));
                }
                LineRef::Text(text) => {
                    self.bookmark.next_line();
                    return Ok(Line::Dialogue(Dialogue::from(
                        &self.speaker,
                        text,
                        self.story,
                        &self.bookmark,
                    )?));
                }
            };
            input = "";
        }
    }

    /// Set state values.
    pub fn set_state(&mut self, statemod: StateMod, value: Value) -> Result<()> {
        self.bookmark.set_value(statemod, value)
    }
    /// Return the state value for the given varname.
    pub fn get_state(&self, varname: &str) -> Result<&Value> {
        self.bookmark.value(varname)
    }

    /// Reads the current line.
    fn readline(&self) -> Result<LineRef<'story>> {
        if self.bookmark.passage().is_empty() {
            return Ok(LineRef::Return);
        }
        if self.bookmark.line() >= self.lines.len() {
            return Err(error!(
                "Invalid line number {} in passage '{}'",
                self.bookmark.line(),
                self.bookmark.passage()
            ));
        };
        Ok(self.lines[self.bookmark.line()])
    }

    /// Returns true if tail call optimization is possible.
    /// This requires that the current line is a return statement, and
    /// that this section has no `on_exit` callback.
    fn can_optimize_tail_call(&self) -> bool {
        if let LineRef::Return = self.lines[self.bookmark.line()] {
            match self.has_on_exit_cmd() {
                Err(_) => false,
                Ok(has_on_exit) => !has_on_exit,
            }
        } else {
            false
        }
    }

    /// Calls the default target for this choices object.
    /// If the default is lines, then we skip all lines in standard choices
    /// to land on the first default embedded passage line.
    fn call_default(&mut self, raw_choices: &RawChoices) -> Result<()> {
        match &raw_choices.default {
            ChoiceTarget::None => Err(error!("No choice target available.")),
            ChoiceTarget::Lines(_lines) => {
                self.bookmark
                    .skip_lines(raw_choices.line_len() - raw_choices.default.line_len() - 1);
                Ok(())
            }
            ChoiceTarget::PassageName(passage_name) => {
                self.call_choice(raw_choices, passage_name.clone())
            }
        }
    }

    /// Calls a choice.
    /// Before calling, it advances the pointer to one step above where choices ends on the stack.
    fn call_choice(&mut self, raw_choices: &RawChoices, passage_name: String) -> Result<()> {
        // Skip to one line before last line of choices.
        // This ensures that the line number on the stack is the next line after this choice.
        self.bookmark.skip_lines(raw_choices.line_len() - 1);
        self.call(passage_name)
    }

    /// Call the configured passage by putting return position on stack.
    /// Goto the passage.
    fn call(&mut self, passage_name: String) -> Result<()> {
        self.bookmark.next_line();

        // Don't push this func onto the stack of the next line is just a return.
        // (Tail call optimization).
        if !self.can_optimize_tail_call() {
            self.bookmark.stack.push(self.bookmark.position().clone());
        }
        self.goto(passage_name)?;
        Ok(())
    }

    /// Repopulates `self` with a list of all valid choices from `raw` in order.
    /// Also repopulates the `choice_to_passage` and `choice_to_line_num` maps.
    fn load_choices(&mut self, raw: &'story RawChoices) -> Result<Choices> {
        let choices = Choices::from_raw(
            &mut self.choice_to_passage,
            &mut self.choice_to_line_num,
            raw,
            &self.bookmark,
        )?;
        Ok(choices)
    }

    /// Loads lines into a single flat array of references and initializes breakpoint stack.
    /// For anything that requires control flow (branches, choices), store the position
    /// we need to jump to afterwards using a `Break(line_num)`.
    fn load_lines(&mut self, lines: &'story [RawLine]) {
        for line in lines {
            self.lines.push(LineRef::from(line));
            match line {
                RawLine::Branches(branches) => {
                    let branch_end = self.lines.len() - 1 + branches.line_len();
                    for (_expression, branch_lines) in &branches.exprs {
                        self.load_lines(branch_lines);
                        self.lines.push(LineRef::Break(branch_end));
                    }
                    // Remove the last break, since it's redundant.
                    self.lines.pop();
                }
                RawLine::Choices(choices) => {
                    let choices_end = self.lines.len() - 1 + choices.line_len();
                    let mut load_target = |target: &'story ChoiceTarget| {
                        if let ChoiceTarget::Lines(lines) = target {
                            self.load_lines(lines);
                            self.lines.push(LineRef::Break(choices_end));
                        }
                    };
                    for (_key, choice) in choices {
                        match choice {
                            RawChoice::Target(target) => load_target(target),
                            RawChoice::Conditional(conditional) => {
                                for (_inner_key, target) in conditional {
                                    load_target(target)
                                }
                            }
                        }
                    }

                    // Remove the last break, since it's redundant.
                    if let Some(LineRef::Break(_)) = self.lines.last() {
                        self.lines.pop();
                    }

                    // Add the default lines if they exist.
                    if let ChoiceTarget::Lines(lines) = &choices.default {
                        self.load_lines(lines)
                    }
                }
                _ => (),
            }
        }
    }

    /// Runs the `onEnter` set command.
    fn run_on_enter(&mut self) -> Result<()> {
        self.story
            .apply_set_commands(|section| section.on_enter(), &mut self.bookmark)
    }

    /// Runs the `onExit` set command.
    fn run_on_exit(&mut self) -> Result<()> {
        self.story
            .apply_set_commands(|section| section.on_exit(), &mut self.bookmark)
    }

    /// Returns true if the current section has an `onExit` command to run.
    fn has_on_exit_cmd(&self) -> Result<bool> {
        let set_commands = self
            .story
            .get_set_commands(|section| section.on_exit(), &self.bookmark)?;
        Ok(!set_commands.is_empty())
    }

    /// Loads the current passage based on the bookmark's position.
    /// Automatically handles updating of namespace.
    pub fn load_passage(&mut self) -> Result<()> {
        let qname = QualifiedName::from(self.bookmark.namespace(), self.bookmark.passage());
        let (namespace, _section, passage) = self.story.passage(&qname)?;
        let (namespace, passage_name) = (namespace.to_string(), qname.name.to_string());
        self.bookmark.update_position(namespace, passage_name);

        self.lines.clear();
        self.load_lines(passage);

        // If lines doesn't end in a return, push a return.
        match self.lines.last() {
            Some(LineRef::Return) => (),
            None | Some(_) => self.lines.push(LineRef::Return),
        }

        // Debug statements.
        // println!("\nLoaded new passage:");
        // for (i, e) in self.lines.iter().enumerate() {
        //     println!("L{}: {:?}", i, e);
        // }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bookmark, Line, Runner, Story};

    #[test]
    fn test_default_bookmark() {
        let mut runner = Runner::init(Bookmark::default(), Story::default(), true).unwrap();
        assert_eq!(runner.next("").unwrap(), Line::End);
    }
}
