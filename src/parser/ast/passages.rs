use pest::iterators::Pairs;

use crate::{parser::Rule, Map, Result, TryFrom};

#[derive(Debug, PartialEq)]
pub enum Line {}

#[derive(Debug, PartialEq)]
pub struct Passage {
    lines: Vec<Line>,
}
impl TryFrom<Pairs<'_, Rule>> for Passage {
    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self> {
        Ok(Self { lines: vec![] })
    }
}

#[derive(Debug, PartialEq)]
pub struct Passages {
    passages: Map<String, Passage>,
}
impl TryFrom<Pairs<'_, Rule>> for Passages {
    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self> {
        Ok(Self {
            passages: Map::new(),
        })
    }
}
