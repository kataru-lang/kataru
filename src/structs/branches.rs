use super::{Bookmark, Conditional, Line};
use crate::error::ParseError;
use crate::traits::Parsable;
use linear_map::LinearMap;

pub trait Branchable {
    fn take(&self, bookmark: &mut Bookmark) -> Result<usize, ParseError>;
    fn length(&self) -> usize;
}

pub type Branches = LinearMap<String, Vec<Line>>;

impl Branchable for Branches {
    /// Evaluates the conditionals in a given branch and takes the first one that evaluates to true.
    fn take(&self, bookmark: &mut Bookmark) -> Result<usize, ParseError> {
        let mut skip_lines = 1;
        for (expression, lines) in self {
            if expression == "else" {
                continue;
            };
            if Conditional::parse(expression)?.eval(bookmark)? {
                break;
            } else {
                skip_lines += flattened_len(lines) + 1;
            }
        }
        bookmark.line += skip_lines;
        Ok(skip_lines)
    }

    fn length(&self) -> usize {
        let mut length = 0;

        for (_expression, branch_lines) in self {
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
