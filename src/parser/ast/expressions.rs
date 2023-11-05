use super::operators::{BinaryOperator, UnaryOperator};
use crate::parser::Rule;
use crate::{Error, Result, TryFrom};
use pest::iterators::{Pair, Pairs};

#[derive(Debug, PartialEq)]
pub struct BinaryExpression {
    lhs: Box<Expression>,
    rhs: Vec<(BinaryOperator, Expression)>,
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

#[derive(Debug, PartialEq)]
pub struct UnaryExpression {
    op: UnaryOperator,
    expr: Box<Expression>,
}
impl TryFrom<Pairs<'_, Rule>> for UnaryExpression {
    fn try_from(mut pairs: Pairs<'_, Rule>) -> Result<Self> {
        Ok(Self {
            op: UnaryOperator::try_from(pairs.next().unwrap().as_rule())?,
            expr: Box::new(Expression::try_from(pairs.next().unwrap())?),
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Number(f64),
    String(String),
    Bool(bool),
    Variable(String),
    BinaryExpression(BinaryExpression),
    UnaryExpression(UnaryExpression),
}
impl TryFrom<Pair<'_, Rule>> for Expression {
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self> {
        Ok(match pair.as_rule() {
            Rule::BinaryExpression => {
                Self::BinaryExpression(BinaryExpression::try_from(pair.into_inner())?)
            }
            Rule::UnaryExpression => {
                Self::UnaryExpression(UnaryExpression::try_from(pair.into_inner())?)
            }
            Rule::Bool => Self::Bool(bool::try_from(pair)?),
            Rule::Number => Self::Number(f64::try_from(pair)?),
            Rule::String => Self::String(String::try_from(pair)?),
            Rule::Variable => Self::Variable(String::try_from(pair)?),
            _ => unreachable!("Invalid rule type: {:?}", pair.as_rule()),
        })
    }
}

impl TryFrom<Pair<'_, Rule>> for bool {
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self> {
        Ok(match pair.as_str() {
            "true" => true,
            "True" => true,
            "false" => false,
            "False" => false,
            _ => return Err(error!("Invalid bool")),
        })
    }
}

impl TryFrom<Pair<'_, Rule>> for String {
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self> {
        Ok(pair.as_str().to_owned())
    }
}

impl TryFrom<Pair<'_, Rule>> for f64 {
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self> {
        Ok(pair
            .as_str()
            .parse::<f64>()
            .map_err(|_| error!("Invalid float returned by grammar: '{}'", pair.as_str()))?)
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::{BinaryExpression, Expression};
    use crate::{
        parser::{ast::operators::BinaryOperator, Rule, StoryParser},
        TryFrom,
    };
    #[test]
    fn test_expression() {
        let mut pairs = StoryParser::parse(Rule::Expression, "$x + 1 - 2").expect("Error parsing.");
        let expression = Expression::try_from(pairs.next().unwrap()).expect("Failed to parse.");
        assert_eq!(
            expression,
            Expression::BinaryExpression(BinaryExpression {
                lhs: Box::new(Expression::Variable("$x".to_string())),
                rhs: vec![
                    (BinaryOperator::Add, Expression::Number(1.0)),
                    (BinaryOperator::Sub, Expression::Number(2.0))
                ]
            })
        )
    }
}
