mod ast;
#[cfg(feature = "lsp")]
pub mod semtokens;
mod span;
use crate::parser::ast::Expression;
use crate::{Result, TryFrom};
use pest::Parser;

/// Pest parser generated from ast/grammar.pest.
#[derive(pest_derive::Parser)]
#[grammar = "parser/kataru.pest"]
struct StoryParser;

impl StoryParser {
    pub fn build_ast(source: &str) -> Result<Expression> {
        let mut pairs = StoryParser::parse(Rule::Expression, source)?;
        Expression::try_from(pairs.next().unwrap())
    }
}
