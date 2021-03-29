use super::{Bookmark, Line};
use crate::{error::Result, Value};
use linear_map::LinearMap;

pub trait Branchable {
    fn take(&self, bookmark: &mut Bookmark) -> Result<usize>;
    fn length(&self) -> usize;
}

pub type Branches = LinearMap<String, Vec<Line>>;

pub fn get_bool_expr(expr: &str) -> &str {
    let if_prefix = "if ";
    let elif_prefix = "elif ";
    if expr.starts_with(if_prefix) {
        &expr[if_prefix.len()..]
    } else if expr.starts_with(elif_prefix) {
        &expr[elif_prefix.len()..]
    } else {
        ""
    }
}

impl Branchable for Branches {
    /// Evaluates the conditionals in a given branch and takes the first one that evaluates to true.
    fn take(&self, bookmark: &mut Bookmark) -> Result<usize> {
        let mut skip_lines = 1; // Skip the initial if line.

        let mut i = 0;
        for (expr, lines) in self {
            i += 1;

            // If we should execute this block
            if expr == "else" || Value::eval_bool_exprs(get_bool_expr(expr), bookmark)? {
                break;
            } else {
                // Skip all contained lines plus the break that's inserted at the end.
                skip_lines += flattened_len(lines);

                // If not the last section in the if block, add an extra skip for the Line::Break.
                if i < self.len() {
                    skip_lines += 1;
                }
            }
        }
        bookmark.position.line += skip_lines;
        Ok(skip_lines)
    }

    fn length(&self) -> usize {
        let mut length = 0;

        for (_expression, branch_lines) in self {
            length += 1 + flattened_len(branch_lines);
        }
        println!("length: {}", length);
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
