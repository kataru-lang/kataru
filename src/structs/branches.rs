use super::{line_len, Bookmark, RawLine};
use crate::{error::Result, Value};
use linear_map::LinearMap;
use serde::{Deserialize, Serialize};

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

impl Branches {
    /// Evaluates the conditionals in a given branch and takes the first one that evaluates to true.
    pub fn take(&self, bookmark: &mut Bookmark) -> Result<usize> {
        let mut skip_lines = 1; // Skip the initial if line.

        let mut i = 0;
        for (expr, lines) in &self.exprs {
            i += 1;

            // If we should execute this block
            if expr == "else" || Value::from_conditional(expr, bookmark)? {
                break;
            } else {
                // Skip all contained lines plus the break that's inserted at the end.
                skip_lines += line_len(lines);

                // If not the last section in the if block, add an extra skip for the Line::Break.
                if i < self.exprs.len() {
                    skip_lines += 1;
                }
            }
        }
        let next_line = bookmark.line() + self.line_len() - skip_lines;
        bookmark.skip_lines(skip_lines);
        Ok(next_line)
    }

    /// A branch is the length of all it's sub-parts, plus one break
    /// line for each expression except for the last expression.
    /// Finally, has one extra line for the branch itself.
    pub fn line_len(&self) -> usize {
        let mut length = self.exprs.len();
        for (_expr, branch_lines) in &self.exprs {
            length += line_len(branch_lines);
        }
        length
    }
}

#[cfg(test)]
mod tests {
    use linear_map::linear_map;

    use super::{Branches, RawLine};

    #[test]
    fn test_branches_length() {
        let branches = Branches {
            exprs: linear_map! {
                "if true".to_string() => vec![
                    RawLine::Text("test".to_string())
                ]
            },
        };
        assert_eq!(branches.line_len(), 2);

        let branches = Branches {
            exprs: linear_map! {
                "if true".to_string() => vec![
                    RawLine::Text("test".to_string())],
                "elif true".to_string() => vec![
                    RawLine::Text("test".to_string())]
            },
        };
        assert_eq!(branches.line_len(), 4);

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
        assert_eq!(branches.line_len(), 3);

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
        assert_eq!(branches.line_len(), 5);
    }
}
