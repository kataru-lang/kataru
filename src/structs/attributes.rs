use super::{Map, QualifiedName};
use crate::error::{Error, Result};
use crate::value::Value;
use crate::Story;
use serde::{Deserialize, Serialize};

pub type Attributes = Vec<AttributedSpan>;
pub type OptionalParams = Map<String, Option<Value>>;

/// Enum representing possible ways to configure an attribute.
/// If a single value, this attribute is a standard valued attribute (e.g. <size=10/>).
/// If it's a map of optional params, then it's a macro that expands to multiple attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeConfig {
    /// This attribute is registered in Kataru, but has no value.
    Value(Value),
    /// Processing an open tag.
    Macro(OptionalParams),
}

/// All attributes that live on the stack must be of this type.
#[derive(Debug, Clone)]
enum AttributeType<'i> {
    /// This attribute is not registered in Kataru and should be ignored.
    Ignored(&'i str),
    /// This attribute is registered in Kataru, but has no value.
    Single(SingleAttributedSpan),
    /// This attribute is actually a macro, containing multiple parameters.
    Macro(&'i str, AttributedSpan),
}
impl<'i> AttributeType<'i> {
    fn parse(text: &str) -> Result<(&str, Option<Value>)> {
        // If a value is contained in this span, split and parse.
        Ok(match text.split_once('=') {
            None => (text, None),
            Some((split_attr, val_str)) => (split_attr, Some(Value::from_yml(val_str)?)),
        })
    }

    /// Construct an attribute type by parsing `text`. Span is started at position `start`.
    /// `story` and `namespace` are used for identifier resolution.
    pub fn from(text: &'i str, start: usize, story: &Story, namespace: &str) -> Result<Self> {
        let (attr, value) = Self::parse(text)?;
        Ok(
            match story.attribute(&QualifiedName::from(namespace, attr)) {
                Err(_) => Self::Ignored(attr),
                Ok(None) => {
                    let mut single = SingleAttributedSpan::new(attr, start)?;
                    single.value = value;
                    Self::Single(single)
                }
                Ok(Some(AttributeConfig::Value(default_value))) => {
                    let mut single = SingleAttributedSpan::new(attr, start)?;
                    single.value = if value.is_some() {
                        value
                    } else {
                        Some(default_value.clone())
                    };
                    Self::Single(single)
                }
                Ok(Some(AttributeConfig::Macro(params))) => {
                    Self::Macro(attr, AttributedSpan::new(start, params.clone()))
                }
            },
        )
    }

    /// Gets the name of this attribute.
    pub fn name(&self) -> &str {
        match self {
            AttributeType::Ignored(last_attr) => last_attr,
            AttributeType::Macro(last_attr, _span) => last_attr,
            AttributeType::Single(span) => &span.name,
        }
    }

    /// Gets the name of this attribute.
    pub fn set_end(&mut self, end: usize) {
        match self {
            AttributeType::Ignored(_) => (),
            AttributeType::Macro(_last_attr, span) => span.end = end,
            AttributeType::Single(span) => span.end = end,
        }
    }
}

/// Trait expressing spans over text.
trait Span {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn same_span<S: Span>(&self, other: &S) -> bool {
        self.start() == other.start() && self.end() == other.end()
    }
}

/// A span of text with a map of attributes and values.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AttributedSpan {
    pub start: usize,
    pub end: usize,
    pub params: Map<String, Option<Value>>,
}
impl AttributedSpan {
    /// Constructs a new attributed span.
    fn new(start: usize, params: Map<String, Option<Value>>) -> Self {
        Self {
            start,
            end: start,
            params,
        }
    }
    /// Merges a single attributed span into this span.
    fn merge(&mut self, span: SingleAttributedSpan) {
        self.params.insert(span.name, span.value);
    }
    /// Merges an attribute macro of params into these params.
    fn merge_with_params(&mut self, params: &OptionalParams) {
        self.params.extend(params.clone().into_iter())
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
impl SingleAttributedSpan {
    fn new(mut text: &str, start: usize) -> Result<Self> {
        let mut value = None;

        // If a value is contained in this span, split and parse.
        match text.split_once('=') {
            None => (),
            Some((split_attr, val_str)) => {
                text = split_attr;
                value = Some(Value::from_yml(val_str)?);
            }
        }

        Ok(Self {
            start: start,
            end: start,
            name: text.to_string(),
            value,
        })
    }
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
pub struct AttributeExtractor<'a, 'i> {
    /// Output: result attributes.
    attributes: Vec<AttributedSpan>,
    /// Output: text without any attributes.
    stripped: String,

    // Input: config.
    story: &'a Story,
    // Input: namespace of current context.
    namespace: &'a str,

    /// Parse state: start of the current token.
    start: usize,
    /// Parse state: the current context for the parsing state machine.
    context: Context,
    /// Parse state: a stack of active attributed spans.
    stack: Vec<AttributeType<'i>>,
}
impl<'a, 'i> AttributeExtractor<'a, 'i> {
    /// Initialize the extractor with defaults.
    /// The stack starts is a default single attributed span on top.
    pub fn new(namespace: &'a str, story: &'a Story) -> Self {
        Self {
            attributes: Attributes::default(),
            namespace,
            story,
            stripped: String::new(),
            start: 0,
            context: Context::Text,
            stack: Vec::new(),
        }
    }

    /// Extracts attributes from a string.
    pub fn extract_attr(
        text: &str,
        namespace: &'a str,
        story: &'a Story,
    ) -> Result<(Attributes, String)> {
        let mut extractor = Self::new(namespace, story);
        extractor.extract(text)?;
        Ok((extractor.attributes, extractor.stripped))
    }

    fn extract(&mut self, text: &'i str) -> Result<()> {
        for (i, c) in text.char_indices() {
            self.consume_next(text, i, c)?;
        }

        // Push remaining text.
        if let Context::Open = self.context {
            // If in Open context, then start is push one step ahead due account for the '<' character.
            self.start -= 1;
        }
        self.stripped.push_str(&text[self.start..]);

        // Handle unmatched tags.
        if let Some(attr_type) = self.stack.pop() {
            return Err(error!("Unmatched tag <{}>", attr_type.name()));
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
    fn finish_attr(&mut self, attr_type: AttributeType, text: &'i str, i: usize) {
        // If this span is actually a macro, inject the macro values instead of the span.
        match attr_type {
            AttributeType::Macro(_name, span) => {
                if let Some(mergeable_span) = self.get_mergeable_span_mut(&span) {
                    mergeable_span.merge_with_params(&span.params)
                } else {
                    self.attributes.push(span)
                }
            }
            AttributeType::Single(span) => {
                if let Some(mergeable_span) = self.get_mergeable_span_mut(&span) {
                    mergeable_span.merge(span)
                } else {
                    self.attributes.push(AttributedSpan::from(span))
                }
            }
            AttributeType::Ignored(_) => {
                let start = match self.context {
                    Context::Close => self.start - "</".len(),
                    Context::SelfClose => self.start - "<".len(),
                    _ => unreachable!(),
                };
                let end = i + ">".len();
                self.stripped.push_str(&text[start..end]);
            }
        }
    }

    /// Constructs an attr type using data from the extractor.
    fn build_attr_type(&self, text: &'i str) -> Result<AttributeType<'i>> {
        AttributeType::from(text, self.stripped.len(), &self.story, &self.namespace)
    }

    fn consume_next(&mut self, text: &'i str, i: usize, c: char) -> Result<()> {
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
                        self.start = i + "/".len();
                        self.context = Context::Close;
                    }
                    // Self-closing tag, e.g. "<tag/>"".
                    else {
                        self.context = Context::SelfClose;
                    }
                }
                // Processing quotes
                else if c == '"' {
                    self.context = Context::Quoted;
                }
                // When done an open tag.
                else if c == '>' {
                    // match self.get_attr_type
                    let attr_type = self.build_attr_type(&text[self.start..i])?;
                    if let AttributeType::Ignored(_) = attr_type {
                        self.stripped
                            .push_str(&text[self.start - "<".len()..i + ">".len()]);
                    }
                    self.stack.push(attr_type);
                    self.start = i + 1;
                    self.context = Context::Text;
                }
            }
            Context::Close => {
                // When done with a close tag, we expect the same tag to be on the top of the stack.
                if c == '>' {
                    let attr = &text[self.start..i];
                    let mut attr_type = match self.stack.pop() {
                        Some(attr_type) => attr_type,
                        None => return Err(error!("Closing tag </{}> had no open tag.", attr)),
                    };

                    let last_attr = attr_type.name();
                    if attr != attr_type.name() {
                        return Err(error!("Tag <{}> was closed before <{}>.", attr, last_attr));
                    }

                    attr_type.set_end(self.stripped.len());
                    self.finish_attr(attr_type, text, i);

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

                let attr_type = self.build_attr_type(&text[self.start..i - "/".len()])?;
                self.finish_attr(attr_type, text, i);
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
    use crate::{Config, Map, Section};

    #[test]
    fn test_extract_attr() {
        let hey_params = hashmap! {
            "sfx".to_string() => Some(Value::String("hey".to_string())),
            "volume".to_string() => Some(Value::Number(10.)),
            "emote".to_string() => Some(Value::String("angry".to_string()))
        };
        let story = Story::from(hashmap! {
            "global".to_string() => Section { config: Config {
                namespace: "global".to_string(),
                attributes: hashmap! {
                    "attr1".to_string() => None,
                    "attr2".to_string() => None,
                    "sfx".to_string() => None,
                    "volume".to_string() => None,
                    "emote".to_string() => None,
                    "hey".to_string() => Some(AttributeConfig::Macro(hey_params.clone()))
                },
                ..Config::default()
            }, passages: Map::new() }
        });

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
                        params: hey_params.clone(),
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
                        params: hey_params.clone(),
                    }],
                    "Test hey.".to_string(),
                )),
            ),
            (
                "Test <b>text</b>.",
                Ok((Vec::new(), "Test <b>text</b>.".to_string())),
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
            ("Test < text.", Ok((Vec::new(), "Test < text.".to_string()))),
        ];

        for (text, expected) in tests {
            let result = AttributeExtractor::extract_attr(text, "global", &story);
            assert_eq!(result, expected);
        }
    }
}
