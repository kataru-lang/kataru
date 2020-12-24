use super::{Conditional, Parsable};
use crate::{Config, Line, ValidationError};
use linear_map::LinearMap;

pub trait Branchable {
    fn take(&self, config: &mut Config) -> Result<(), ValidationError>;
}

pub type Branches = LinearMap<String, Vec<Line>>;

pub fn branch_len(lines: &[Line]) -> usize {
    let mut length = lines.len();
    for line in lines {
        if let Line::Branches(branches) = line {
            for (_expression, branch_lines) in branches {
                length += branch_len(branch_lines);
            }
        }
    }
    length
}
impl Branchable for Branches {
    /// Evaluates the conditionals in a given branch and takes the first one that evaluates to true.
    fn take(&self, config: &mut Config) -> Result<(), ValidationError> {
        let mut skip_lines = 1;
        for (expression, lines) in self {
            if expression == "else" {
                continue;
            };
            if Conditional::parse(expression)?.eval(&config.state)? {
                break;
            } else {
                skip_lines += branch_len(lines);
            }
        }
        config.line += skip_lines;
        Ok(())
    }
}
