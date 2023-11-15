use pest::iterators::{Pair, Pairs};

use super::Rule;

pub const LEGEND: &[lsp_types::SemanticTokenType] = &[
    lsp_types::SemanticTokenType::FUNCTION,
    lsp_types::SemanticTokenType::VARIABLE,
    lsp_types::SemanticTokenType::STRING,
    lsp_types::SemanticTokenType::COMMENT,
    lsp_types::SemanticTokenType::NUMBER,
    lsp_types::SemanticTokenType::KEYWORD,
    lsp_types::SemanticTokenType::OPERATOR,
    lsp_types::SemanticTokenType::PARAMETER,
];

#[derive(PartialEq, Debug)]
pub enum SemanticTokenType {
    Function = 0,
    Variable = 1,
    String = 2,
    Comment = 3,
    Number = 4,
    Keyword = 5,
    Operator = 6,
    Parameter = 7,
}
impl SemanticTokenType {
    pub fn try_from(rule: Rule) -> Option<Self> {
        Some(match rule {
            Rule::COMMENT => SemanticTokenType::Comment,
            Rule::Add | Rule::Sub | Rule::Mul | Rule::Div => SemanticTokenType::Operator,
            Rule::Variable => SemanticTokenType::Variable,
            Rule::Number => SemanticTokenType::Number,
            Rule::QuotedString | Rule::StringId | Rule::StringLiteral => SemanticTokenType::String,
            Rule::namespace
            | Rule::state
            | Rule::set
            | Rule::input
            | Rule::r#else
            | Rule::call
            | Rule::r#if
            | Rule::And
            | Rule::Or
            | Rule::Not
            | Rule::default
            | Rule::global => SemanticTokenType::Keyword,
            _ => return None,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct SemanticToken {
    pub start: usize,
    pub end: usize,
    pub token_type: SemanticTokenType,
}
impl SemanticToken {
    pub fn try_from(pair: &Pair<'_, Rule>) -> Option<Self> {
        if let Some(token_type) = SemanticTokenType::try_from(pair.as_rule()) {
            Some(Self {
                start: pair.as_span().start(),
                end: pair.as_span().end(),
                token_type,
            })
        } else {
            None
        }
    }
}

fn tokenize_recursive(pairs: Pairs<'_, Rule>, tokens: &mut Vec<SemanticToken>) {
    for pair in pairs {
        if let Some(token) = SemanticToken::try_from(&pair) {
            tokens.push(token);
        }
        tokenize_recursive(pair.into_inner(), tokens);
    }
}
pub fn tokenize(pairs: Pairs<'_, Rule>) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    tokenize_recursive(pairs, &mut tokens);
    tokens
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::parser::semtokens::{tokenize, SemanticToken, SemanticTokenType};
    use crate::parser::{Rule, StoryParser};
    #[test]
    fn test_tokenize() {
        let pairs = StoryParser::parse(Rule::Expression, "$x + 1 - 2").expect("Error parsing.");
        assert_eq!(
            tokenize(pairs),
            vec![
                SemanticToken {
                    start: 0,
                    end: 2,
                    token_type: SemanticTokenType::Variable
                },
                SemanticToken {
                    start: 3,
                    end: 4,
                    token_type: SemanticTokenType::Operator
                },
                SemanticToken {
                    start: 5,
                    end: 6,
                    token_type: SemanticTokenType::Number
                },
                SemanticToken {
                    start: 7,
                    end: 8,
                    token_type: SemanticTokenType::Operator
                },
                SemanticToken {
                    start: 9,
                    end: 10,
                    token_type: SemanticTokenType::Number
                }
            ]
        )
    }
}
