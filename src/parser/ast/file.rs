use super::operators::{BinaryOperator, UnaryOperator};
use crate::parser::{span::Spanned, Rule};
use crate::{Error, Result, TryFrom};
use pest::iterators::{Pair, Pairs};

#[derive(Debug, PartialEq)]
pub struct NamespaceHeader {
    namespace: String,
}

#[derive(Debug, PartialEq)]
pub struct Headers {
    state: StateHeader,
    namespace: NamespaceHeader,
}
#[derive(Debug, PartialEq)]
pub struct File {
    headers: Headers,
}
impl TryFrom<Pairs<'_, Rule>> for BinaryExpression {
    fn try_from(mut pairs: Pairs<'_, Rule>) -> Result<Self> {
        let lhs = Box::new(Expression::try_from(pairs.next().unwrap())?);
        let mut rhs = vec![];
        rhs.reserve(pairs.len() / 2);
        while pairs.len() > 0 {
            let op = BinaryOperator::try_from(pairs.next().unwrap().as_rule())?;
            let operand = Expression::try_from(pairs.next().unwrap())?;
            rhs.push((op, operand))
        }
        Ok(Self { lhs, rhs })
    }
}
