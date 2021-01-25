use crate::structs::{
    Bookmark, Branchable, Choice, Choices, Dialogue, Line, Passage, QualifiedName, StateUpdatable,
    Story, StoryGetters,
};
use crate::vars::replace_vars;

pub struct Runner<'r> {
    pub bookmark: &'r mut Bookmark,
    pub story: &'r Story,
    pub line: usize,
    pub passage: &'r Passage,
    lines: Vec<&'r Line>,
    breaks: Vec<usize>,
    choices: Choices,
}

impl<'r> Runner<'r> {
    pub fn new(bookmark: &'r mut Bookmark, story: &'r Story) -> Self {
        // Flatten dialogue lines
        let passage = &story
            .passage(&QualifiedName::from(&bookmark.namespace, &bookmark.passage))
            .unwrap();
        let mut runner = Self {
            bookmark,
            story,
            line: 0,
            lines: vec![],
            passage,
            breaks: vec![],
            choices: Choices::default(),
        };
        runner.load_lines(passage);
        runner.init_breaks();
        runner
    }

    /// Initialize the line break stack.
    /// Loop through each line in the flattened array until current line
    /// number is reached.
    /// Each time a branch is detected, push the end of the branch on the break stack.
    fn init_breaks(&mut self) {
        for (line_num, line) in self.lines.iter().enumerate() {
            if line_num >= self.bookmark.line {
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

    /// Goto a given `passage_name`.
    fn goto(&mut self, passage_name: &str) {
        // `passage_name` could be:
        // 1) a local name (unquallified), in which case namespace stays the same.
        // 2) a qualified name pointing to another section, in which case we switch namespace.
        // 3) a global name, in which we must changed namespace to root.
        let qname = QualifiedName::from(&self.bookmark.namespace, passage_name);
        self.passage = match self
            .story
            .get(&qname.namespace)
            .unwrap()
            .passage(&qname.name)
        {
            Some(passage) => {
                if !qname.namespace.is_empty() {
                    self.bookmark.namespace = qname.namespace;
                }
                passage
            }
            None => {
                self.bookmark.namespace = "".to_string();
                self.story.get("").unwrap().passage(&qname.name).unwrap()
            }
        };

        self.bookmark.passage = qname.name;
        self.bookmark.line = 0;

        self.lines = vec![];
        self.breaks = vec![];
        self.load_lines(self.passage);
    }

    /// Processes a line.
    /// Returning &Line::Continue signals to `next()` that another line should be processed
    /// before returning a line to the user.
    fn process_line(&mut self, input: &str, line: &'r Line) -> Line {
        match line {
            // When a choice is encountered, it should first be returned for display.
            // Second time its encountered,
            Line::Choices(choices) => {
                // If empty input, chocies are being returned for display.
                if input.is_empty() {
                    self.choices = choices.get_valid(&self.bookmark);
                    Line::Choices(self.choices.clone())
                } else if self.choices.choices.contains_key(input) {
                    if let Some(Choice::PassageName(passage_name)) =
                        self.choices.choices.remove(input)
                    {
                        self.goto(&passage_name);
                    }
                    Line::Continue
                } else {
                    Line::InvalidChoice
                }
            }
            Line::Branches(branches) => {
                let skipped_len = branches.take(&mut self.bookmark).unwrap();
                let branch_len = branches.length();
                self.breaks
                    .push(self.bookmark.line + branch_len - skipped_len);
                Line::Continue
            }
            Line::Goto(goto) => {
                self.goto(&goto.goto);
                Line::Continue
            }
            Line::Break => {
                let last_break = self.breaks.pop();
                self.bookmark.line = match last_break {
                    Some(line_num) => line_num,
                    None => 0,
                };
                Line::Continue
            }
            Line::Cmd(_) => {
                self.bookmark.line += 1;
                line.clone()
            }
            Line::SetCmd(set) => {
                let root_sets = self.bookmark.state().update(&set.set).unwrap();
                self.bookmark.root_state().update(&root_sets).unwrap();
                self.bookmark.line += 1;
                Line::Continue
            }
            Line::Dialogue(dialogue) => {
                let mut replaced_dialogue = Dialogue::default();
                for (character, text) in dialogue {
                    replaced_dialogue
                        .insert(character.to_string(), replace_vars(text, self.bookmark));
                }
                self.bookmark.line += 1;
                Line::Dialogue(replaced_dialogue)
            }
            Line::Text(text) => {
                self.bookmark.line += 1;
                Line::Text(replace_vars(text, self.bookmark))
            }
            Line::Continue => {
                self.bookmark.line += 1;
                Line::Continue
            }
            Line::End => Line::End,
            Line::Error => Line::Error,
            Line::InvalidChoice => Line::InvalidChoice,
        }
    }

    /// If the current configuration points to a valid line, processes the line.
    fn process(&mut self, input: &str) -> Line {
        if self.bookmark.line >= self.lines.len() {
            Line::Error
        } else {
            self.process_line(input, self.lines[self.bookmark.line])
        }
    }

    /// Gets the next dialogue line from the story based on the user's input.
    /// Internally, a single call to `next()` may result in multiple lines being processed,
    /// i.e. when a choice is being made.
    pub fn next(&mut self, input: &str) -> Line {
        let mut line = self.process(input);
        while line == Line::Continue {
            line = self.process("");
        }
        line
    }
}
