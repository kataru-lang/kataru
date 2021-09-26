use super::Map;
use crate::error::{Error, Result};
use crate::value::Value;
use serde::{Deserialize, Serialize};

pub type Attributes = Vec<AttributedSpan>;
pub type OptionParams = Map<String, Option<Value>>;

/// Trait expressing spans over text.
trait Span {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn same_span<S: Span>(&self, other: &S) -> bool {
        self.start() == other.start() && self.end() == other.end()
    }
}

/// A span of text with a map of attributes and values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributedSpan {
    pub start: usize,
    pub end: usize,
    pub params: Map<String, Option<Value>>,
}
impl AttributedSpan {
    fn merge(&mut self, span: SingleAttributedSpan) {
        self.params.insert(span.name, span.value);
    }
}
impl From<SingleAttributedSpan> for AttributedSpan {
    fn from(span: SingleAttributedSpan) -> Self {
        let mut params = Map::<String, Option<Value>>::new();
        params.insert(span.name, span.value);
        Self {
            start: span.start,
            end: span.end,
            params: params,
        }
    }
}
impl Span for AttributedSpan {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
}

/// A single span with a name and value pair.
#[derive(Debug, Clone, Default)]
struct SingleAttributedSpan {
    pub start: usize,
    pub end: usize,
    pub name: String,
    pub value: Option<Value>,
}
impl Span for SingleAttributedSpan {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
}

/// While iterating, keep track of what context we are in and where this context begins.
#[derive(Debug, Clone)]
enum Context {
    /// Parsing untagged text.
    Text,
    /// Processing an open tag.
    Open,
    /// Processing a close tag.
    Close,
    /// Process a self closing tag.
    SelfClose,
    /// Processing text in quotes.
    Quoted,
    /// Processing an escaped character inside of quotes.
    Escaped,
}

/// Utility struct for extracting attributes from text.
/// Algorithm:
///   Loop through each character of the string. When we see a valid open tag,
///   Push a new tag onto the stack. When the tag closes, pop it off.
///
/// Notes:
///  To get the current start of a span, read self.stripped.len() when the tag is opened.
///  Similarly we use self.stripped.len() when the tag is closed, since the contained
///  text will have been appended into self.result
///
/// When we process a value, we know we are inside of the context of an open tag.
/// Therefore we don't need to keep track of context in the stack.
#[derive(Debug, Clone)]
pub struct AttributeExtractor<'a> {
    /// Output: result attributes.
    attributes: Vec<AttributedSpan>,
    /// Output: text without any attributes.
    stripped: String,

    // Input: config.
    config: &'a Map<String, Option<OptionParams>>,

    /// Parse state: start of the current token.
    start: usize,
    /// Parse state: the current context for the parsing state machine.
    context: Context,
    /// Parse state: a stack of active attributed spans.
    stack: Vec<SingleAttributedSpan>,
}
impl<'a> AttributeExtractor<'a> {
    /// Initialize the extractor with defaults.
    /// The stack starts is a default single attributed span on top.
    pub fn new(config: &'a Map<String, Option<OptionParams>>) -> Self {
        Self {
            attributes: Attributes::default(),
            config,
            stripped: String::new(),
            start: 0,
            context: Context::Text,
            stack: Vec::new(),
        }
    }

    /// Extracts attributes from a string.
    pub fn extract_attr(
        text: &str,
        attr_config: &'a Map<String, Option<OptionParams>>,
    ) -> Result<(Attributes, String)> {
        let mut extractor = Self::new(attr_config);
        extractor.extract(text)?;
        Ok((extractor.attributes, extractor.stripped))
    }

    fn extract(&mut self, text: &str) -> Result<()> {
        for (i, c) in text.char_indices() {
            self.consume_next(text, i, c)?;
        }

        // Push remaining text.
        if let Context::Open = self.context {
            // If in Open context, then start is push one step ahead due account for the '<' character.
            self.start -= 1;
        }
        self.stripped.push_str(&text[self.start..]);

        if let Some(span) = self.stack.pop() {
            return Err(error!("Unmatched tag <{}>", span.name));
        }
        Ok(())
    }

    /// If the last span has the same start and end as this span, return mut ref to it.
    fn get_mergeable_span_mut<S: Span>(&mut self, span: &S) -> Option<&mut AttributedSpan> {
        if let Some(added_span) = self.attributes.last_mut() {
            if added_span.same_span(span) {
                return Some(added_span);
            }
        }
        None
    }

    /// When pushing a span, to keep the returned data structure more consice
    /// we merge params over the same span into the same struct.
    fn push_span(&mut self, span: SingleAttributedSpan) {
        // If this span is actually a macro, inject the macro values instead of the span.
        if let Some(Some(params)) = self.config.get(&span.name) {
            if let Some(mergeable_span) = self.get_mergeable_span_mut(&span) {
                return mergeable_span.merge(span);
            } else {
                return self.attributes.push(AttributedSpan {
                    start: span.start,
                    end: span.end,
                    params: params.clone(),
                });
            }
        }
        if let Some(mergeable_span) = self.get_mergeable_span_mut(&span) {
            mergeable_span.merge(span)
        } else {
            self.attributes.push(AttributedSpan::from(span))
        }
    }

    /// Initializes a span from text.
    fn init_span(&self, mut attr: &str) -> Result<SingleAttributedSpan> {
        let mut value = None;

        // If a value is contained in this span, split and parse.
        match attr.split_once('=') {
            None => (),
            Some((split_attr, val_str)) => {
                attr = split_attr;
                value = Some(Value::from_yml(val_str)?);
            }
        }

        Ok(SingleAttributedSpan {
            start: self.stripped.len(),
            end: self.stripped.len(),
            name: attr.to_string(),
            value,
        })
    }

    fn consume_next(&mut self, text: &str, i: usize, c: char) -> Result<()> {
        match self.context {
            Context::Text => {
                // If we reach an open tag.
                if c == '<' {
                    // Push text into the results.
                    self.stripped.push_str(&text[self.start..i]);
                    self.start = i + "<".len();
                    self.context = Context::Open;
                }
            }
            Context::Open => {
                // '/' character can be interpreted as closing tag or self-closing tag.
                if c == '/' {
                    // Closing tag, e.g. "</tag>"".
                    if self.start == i {
                        println!("Closing tag!");
                        self.start = i + "/".len();
                        self.context = Context::Close;
                    }
                    // Self-closing tag, e.g. "<tag/>"".
                    else {
                        println!("Self-closing tag!");
                        self.context = Context::SelfClose;
                    }
                }
                // Processing quotes
                else if c == '"' {
                    self.context = Context::Quoted;
                }
                // When done an open tag.
                else if c == '>' {
                    let attr = &text[self.start..i];
                    let span = self.init_span(attr)?;
                    if self.config.contains_key(&span.name) {
                        self.stack.push(span);
                    } else {
                        self.stripped
                            .push_str(&text[self.start - "<".len()..i + ">".len()]);
                    }
                    self.start = i + 1;
                    self.context = Context::Text;
                }
            }
            Context::Close => {
                // When done a close tag, we expect the same tag to be on the top of the stack.
                if c == '>' {
                    let attr = &text[self.start..i];
                    if self.config.contains_key(attr) {
                        if let Some(mut span) = self.stack.pop() {
                            if span.name != attr {
                                return Err(error!(
                                    "Tag <{}> was closed before <{}>.",
                                    attr, span.name
                                ));
                            }
                            // This is a valid closing tag, so update the span and commit.
                            span.end = self.stripped.len();
                            self.push_span(span)
                        } else {
                            return Err(error!("Closing tag </{}> had no open tag.", attr));
                        }
                    } else {
                        self.stripped
                            .push_str(&text[self.start - "</".len()..i + ">".len()]);
                    }

                    self.start = i + 1;
                    self.context = Context::Text;
                }
            }
            Context::SelfClose => {
                // Self close must immediately end.
                if c != '>' {
                    return Err(error!(
                        "Self-closing tag {} must immediately close.",
                        &text[self.start..i]
                    ));
                }

                let attr = &text[self.start..i - "/".len()];
                let span = self.init_span(attr)?;
                if self.config.contains_key(&span.name) {
                    self.push_span(span);
                } else {
                    self.stripped
                        .push_str(&text[self.start - "<".len()..i + ">".len()]);
                }
                self.start = i + ">".len();
                self.context = Context::Text;
            }
            Context::Quoted => {
                if c == '\\' {
                    self.context = Context::Escaped;
                } else if c == '"' {
                    self.context = Context::Open;
                }
            }
            Context::Escaped => self.context = Context::Quoted,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Map;

    #[test]
    fn test_extract_attr() {
        let attrs: Map<String, Option<OptionParams>> = hashmap! {
            "attr1".to_string() => None,
            "attr2".to_string() => None,
            "sfx".to_string() => None,
            "volume".to_string() => None,
            "emote".to_string() => None,
            "hey".to_string() => Some(hashmap! {
                "sfx".to_string() => Some(Value::String("hey".to_string())),
                "volume".to_string() => Some(Value::Number(10.)),
                "emote".to_string() => Some(Value::String("angry".to_string()))
            })
        };

        let tests: Vec<(&str, Result<(Attributes, String)>)> = vec![
            (
                "Test <attr1>text</attr1>.",
                Ok((
                    vec![AttributedSpan {
                        start: 5,
                        end: 9,
                        params: hashmap! {"attr1".to_string() => None},
                    }],
                    "Test text.".to_string(),
                )),
            ),
            (
                "Test <hey/>hey.",
                Ok((
                    vec![AttributedSpan {
                        start: 5,
                        end: 5,
                        params: attrs["hey"].as_ref().unwrap().clone(),
                    }],
                    "Test hey.".to_string(),
                )),
            ),
            (
                r#"Test <sfx="hey"/><volume=10/><emote="angry"/>hey."#,
                Ok((
                    vec![AttributedSpan {
                        start: 5,
                        end: 5,
                        params: attrs["hey"].as_ref().unwrap().clone(),
                    }],
                    "Test hey.".to_string(),
                )),
            ),
            (
                "Test <b>text</b>.",
                Ok((Attributes::new(), "Test <b>text</b>.".to_string())),
            ),
            (
                "Test </attr1>text</attr1>.",
                Err(Error::Generic(
                    "Closing tag </attr1> had no open tag.".to_string(),
                )),
            ),
            (
                "Test <attr1>text.",
                Err(Error::Generic("Unmatched tag <attr1>".to_string())),
            ),
            (
                "Test < text.",
                Ok((Attributes::new(), "Test < text.".to_string())),
            ),
        ];

        for (text, expected) in tests {
            let result = AttributeExtractor::extract_attr(text, &attrs);
            println!("{:?}", result);
            assert_eq!(result, expected);
        }
    }
}
