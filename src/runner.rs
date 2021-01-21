use crate::structs::resolve_namespace;
use crate::vars::replace_vars;
use crate::{Bookmark, Branchable, Line, Passage, StateUpdatable, Story, StoryGetters};

pub struct Runner<'r> {
    pub bookmark: &'r mut Bookmark,
    pub story: &'r Story,
    pub line: usize,
    pub passage: &'r Passage,
    lines: Vec<&'r Line>,
    breaks: Vec<usize>,
}

impl<'r> Runner<'r> {
    pub fn new(bookmark: &'r mut Bookmark, story: &'r Story) -> Self {
        // Flatten dialogue lines
        let passage = &story
            .passage(&bookmark.namespace, &bookmark.passage)
            .0
            .unwrap();
        let mut runner = Self {
            bookmark,
            story,
            line: 0,
            lines: vec![],
            passage,
            breaks: vec![],
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

    fn resolve_namespace(&self, name: &str) -> (String, String) {
        let (new_namespace, base_name) = resolve_namespace(&self.bookmark.namespace, name);
        (new_namespace.to_string(), base_name.to_string())
    }

    /// Goto a given `passage_name`.
    fn goto(&mut self, passage_name: &str) {
        // Get the real passage name
        let (new_namespace, base_name) = self.resolve_namespace(passage_name);
        println!("new_namespace: {}, base_name: {}", new_namespace, base_name);

        let (passage_opt, is_root) = self.story.passage(&new_namespace, &base_name);
        self.passage = passage_opt.unwrap();

        // If not currently in root namespace, but passage resolution fell back to use root namespace,
        // then set the namespace to root.
        if self.bookmark.namespace != "" && is_root {
            self.bookmark.namespace = "".to_string();
        } else {
            self.bookmark.namespace = new_namespace;
        }
        self.bookmark.passage = base_name;
        self.bookmark.line = 0;

        self.lines = vec![];
        self.breaks = vec![];
        self.load_lines(self.passage);
    }

    /// Processes a line.
    /// Returning &Line::Continue signals to `next()` that another line should be processed
    /// before returning a line to the user.
    fn process_line(&mut self, input: &str, line: &'r Line) -> &'r Line {
        match line {
            // When a choice is encountered, it should first be returned for display.
            // Second time its encountered,
            Line::SetCmd(set) => {
                let root_sets = self.bookmark.state().update(&set.set).unwrap();
                self.bookmark.root_state().update(&root_sets).unwrap();
                self.bookmark.line += 1;
                &Line::Continue
            }
            Line::Choices(choices) => {
                if choices.choices.contains_key(input) {
                    self.goto(&choices.choices[input]);
                    &Line::Continue
                } else if input.is_empty() {
                    line
                } else {
                    &Line::InvalidChoice
                }
            }
            Line::Branches(branches) => {
                let skipped_len = branches.take(&mut self.bookmark).unwrap();
                let branch_len = branches.length();
                self.breaks
                    .push(self.bookmark.line + branch_len - skipped_len);
                &Line::Continue
            }
            Line::Goto(goto) => {
                self.goto(&goto.goto);
                &Line::Continue
            }
            Line::Break => {
                let last_break = self.breaks.pop();
                self.bookmark.line = match last_break {
                    Some(line_num) => line_num,
                    None => 0,
                };
                &Line::Continue
            }
            _ => {
                // For all others, progress to the next dialog line.
                self.bookmark.line += 1;
                line
            }
        }
    }

    /// If the current configuration points to a valid line, processes the line.
    fn process(&mut self, input: &str) -> Option<&'r Line> {
        if self.bookmark.line >= self.lines.len() {
            None
        } else {
            Some(self.process_line(input, self.lines[self.bookmark.line]))
        }
    }

    /// Gets the next dialogue line from the story based on the user's input.
    /// Internally, a single call to `next()` may result in multiple lines being processed,
    /// i.e. when a choice is being made.
    pub fn next(&mut self, input: &str) -> Option<Line> {
        let mut line = self.process(input)?;
        while line == &Line::Continue {
            line = self.process("")?;
        }

        // Copy the current line so we can modify it for variable replacement.
        let mut cloned_line: Line = line.clone();
        match &mut cloned_line {
            Line::Dialogue(dialogue) => {
                for (_character, text) in dialogue.iter_mut() {
                    *text = replace_vars(text, self.bookmark).trim().to_string();
                }
            }
            Line::Text(text) => {
                *text = replace_vars(text, self.bookmark).trim().to_string();
            }
            _ => (),
        };
        Some(cloned_line)
    }
}
