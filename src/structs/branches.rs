use super::{Bookmark, RawLine};
use crate::{error::Result, Value};
use linear_map::LinearMap;
use serde::{Deserialize, Serialize};

pub trait Branchable {
    fn take(&self, bookmark: &mut Bookmark) -> Result<usize>;
    fn len(&self) -> usize;
}

#[derive(Deserialize)]
pub struct BranchesShadow {
    #[serde(flatten)]
    exprs: LinearMap<String, Vec<RawLine>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "BranchesShadow")]
pub struct Branches {
    #[serde(flatten)]
    pub exprs: LinearMap<String, Vec<RawLine>>,
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
        bookmark.skip_lines(skip_lines);
        Ok(skip_lines)
    }

    /// A branch has one line containing the branch,
    /// plus one break for each consecutive expression,
    /// plus the length of all of its contained lines.
    fn len(&self) -> usize {
        let mut length = self.exprs.len();
        for (_expr, branch_lines) in &self.exprs {
            length += flattened_len(branch_lines);
        }
        length
    }
}

/// All lines take up 1 except for branches,
/// which need their length recursively computed.
fn flattened_len(lines: &[RawLine]) -> usize {
    let mut length = 0;
    for line in lines {
        if let RawLine::Branches(branches) = line {
            length += branches.len();
        } else {
            length += 1
        }
    }
    length
}

#[cfg(test)]
mod tests {
    use linear_map::linear_map;

    use super::{Branchable, Branches, RawLine};

    #[test]
    fn test_branches_length() {
        let branches = Branches {
            exprs: linear_map! {
                "if true".to_string() => vec![
                    RawLine::Text("test".to_string())
                ]
            },
        };
        assert_eq!(branches.len(), 2);

        let branches = Branches {
            exprs: linear_map! {
                "if true".to_string() => vec![
                    RawLine::Text("test".to_string())],
                "elif true".to_string() => vec![
                    RawLine::Text("test".to_string())]
            },
        };
        assert_eq!(branches.len(), 4);

        let branches = Branches {
            exprs: linear_map! {
                "if true".to_string() => vec![
                    RawLine::Branches(Branches {
                        exprs: linear_map! {
                            "if true".to_string() => vec![
                                RawLine::Text("test".to_string())
                                ]
                        },
                    })
                ]
            },
        };
        assert_eq!(branches.len(), 3);

        let branches = Branches {
            exprs: linear_map! {
                "if true".to_string() => vec![
                    RawLine::Branches(Branches {
                        exprs: linear_map! {
                            "if true".to_string() => vec![
                                RawLine::Text("test".to_string())],
                            "elif true".to_string() => vec![
                                RawLine::Text("test".to_string())]
                        },
                    })
                ]
            },
        };
        assert_eq!(branches.len(), 5);
    }
}
