use super::expressions::BinaryExpression;
use super::operators::{BinaryOperator, UnaryOperator};
use super::passages::Passages;
use super::Expression;
use crate::parser::{span::Spanned, Rule};
use crate::{Error, Map, Result, TryFrom};
use pest::iterators::{Pair, Pairs};

#[derive(Debug, PartialEq)]
pub struct NamespaceHeader {
    namespace: String,
}
impl TryFrom<Pairs<'_, Rule>> for NamespaceHeader {
    fn try_from(mut pairs: Pairs<'_, Rule>) -> Result<Self> {
        pairs.next(); // Skip namespace keyword.
        Ok(Self {
            namespace: pairs.next().unwrap().as_str().to_owned(),
        })
    }
}
#[derive(Debug, PartialEq)]
pub struct StateHeader {
    vars: Map<String, Expression>,
}
impl TryFrom<Pairs<'_, Rule>> for StateHeader {
    fn try_from(mut pairs: Pairs<'_, Rule>) -> Result<Self> {
        Ok(Self { vars: Map::new() })
    }
}

#[derive(Debug, PartialEq)]
pub struct Headers {
    state: StateHeader,
    namespace: NamespaceHeader,
}
impl TryFrom<Pairs<'_, Rule>> for Headers {
    fn try_from(mut pairs: Pairs<'_, Rule>) -> Result<Self> {
        Ok(Self {
            state: StateHeader::try_from(pairs.next().unwrap().into_inner())?,
            namespace: NamespaceHeader::try_from(pairs.next().unwrap().into_inner())?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct File {
    headers: Headers,
    passages: Passages,
}
impl TryFrom<Pairs<'_, Rule>> for File {
    fn try_from(mut pairs: Pairs<'_, Rule>) -> Result<Self> {
        Ok(Self {
            headers: Headers::try_from(pairs.next().unwrap().into_inner())?,
            passages: Passages::try_from(pairs.next().unwrap().into_inner())?,
        })
    }
}
