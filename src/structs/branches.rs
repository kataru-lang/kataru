use super::{Bookmark, Line};
use crate::{error::Result, Value};
use linear_map::LinearMap;
use serde::{Deserialize, Serialize};

pub trait Branchable {
    fn take(&self, bookmark: &mut Bookmark) -> Result<usize>;
    fn length(&self) -> usize;
}

#[derive(Deserialize)]
pub struct BranchesShadow {
    #[serde(flatten)]
    exprs: LinearMap<String, Vec<Line>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "BranchesShadow")]
pub struct Branches {
    #[serde(flatten)]
    pub exprs: LinearMap<String, Vec<Line>>,
}

impl std::convert::TryFrom<BranchesShadow> for Branches {
    type Error = &'static str;
    fn try_from(shadow: BranchesShadow) -> std::result::Result<Self, Self::Error> {
        for (key, _val) in &shadow.exprs {
            if !(key.starts_with("if ") || key.starts_with("elif ") || key.starts_with("else")) {
                return Err("Invalid.");
            }
        }
        Ok(Self {
            exprs: shadow.exprs,
        })
    }
}

impl Branchable for Branches {
    /// Evaluates the conditionals in a given branch and takes the first one that evaluates to true.
    fn take(&self, bookmark: &mut Bookmark) -> Result<usize> {
        let mut skip_lines = 1; // Skip the initial if line.

        let mut i = 0;
        for (expr, lines) in &self.exprs {
            i += 1;

            // If we should execute this block
            if expr == "else" || Value::from_conditional(expr, bookmark)? {
                break;
            } else {
                // Skip all contained lines plus the break that's inserted at the end.
                skip_lines += flattened_len(lines);

                // If not the last section in the if block, add an extra skip for the Line::Break.
                if i < self.exprs.len() {
                    skip_lines += 1;
                }
            }
        }
        bookmark.position.line += skip_lines;
        Ok(skip_lines)
    }

    fn length(&self) -> usize {
        let mut length = 0;

        for (_expr, branch_lines) in &self.exprs {
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
