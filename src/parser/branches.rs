use super::{Conditional, Parsable};
use crate::{Config, Line, ValidationError};
use linear_map::LinearMap;

pub trait Branchable {
    fn take(&self, config: &mut Config) -> Result<usize, ValidationError>;
    fn length(&self) -> usize;
}

pub type Branches = LinearMap<String, Vec<Line>>;

impl Branchable for Branches {
    /// Evaluates the conditionals in a given branch and takes the first one that evaluates to true.
    fn take(&self, config: &mut Config) -> Result<usize, ValidationError> {
        let mut skip_lines = 1;
        for (expression, lines) in self {
            if expression == "else" {
                continue;
            };
            if Conditional::parse(expression)?.eval(&config.state)? {
                break;
            } else {
                skip_lines += flattened_len(lines) + 1;
            }
        }
        config.line += skip_lines;
        Ok(skip_lines)
    }

    fn length(&self) -> usize {
        let mut length = 0;

        // Every branch except for the first branch has an extra break appended.
        let mut branches_it = self.iter();
        if let Some((_expression, branch_lines)) = branches_it.next() {
            length += flattened_len(branch_lines);
        }
        for (_expression, branch_lines) in branches_it {
            length += 1 + flattened_len(branch_lines);
        }
        length
    }
}

fn flattened_len(lines: &[Line]) -> usize {
    let mut length = lines.len();
    for line in lines {
        if let Line::Branches(branches) = line {
            length += branches.length();
        }
    }
    length
}
