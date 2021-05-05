use super::Map;
use crate::error::{Error, Result};

pub type Attributes = Map<String, Vec<usize>>;

/// Extracts attributes from a string.
pub fn extract_attr(
    text: &str,
    attrs: &Map<String, Option<String>>,
) -> Result<(Attributes, String)> {
    let mut attributes = Attributes::new();
    let mut result = String::new();

    // While iterating, keep track of what context we are in and where this context begins.
    enum Context {
        Text,
        Open,
        Close,
    }
    let mut context = Context::Text;
    let mut start: usize = 0;

    /// Utility method to finish parsing a tag and run error checking.
    fn finish_tag(
        result: &mut String,
        attributes: &mut Attributes,
        closed: bool,
        attr: &str,
        attrs: &Map<String, Option<String>>,
    ) -> Result<()> {
        // If this is an invalid tag, ignore and push to results.
        if !attrs.contains_key(attr) {
            result.push_str(if closed { "</" } else { "<" });
            result.push_str(attr);
            result.push_str(">");
            return Ok(());
        }

        // Save the position in the result string that this tag starts on.
        let positions = attributes.entry(attr.to_string()).or_insert(vec![]);

        if closed && positions.len() % 2 != 1 {
            return Err(error!("Invalid closing tag </{}>", attr));
        }
        positions.push(result.len());
        Ok(())
    }

    for (i, c) in text.chars().enumerate() {
        match context {
            Context::Text => {
                // If we reach an open tag.
                if c == '<' {
                    // Push text into the results.
                    result.push_str(&text[start..i]);
                    start = i + 1;
                    context = Context::Open;
                }
            }
            Context::Open => {
                // If this tag is actually a close tag.
                if c == '/' {
                    start = i + 1;
                    context = Context::Close;
                }
                // When done an open tag.
                if c == '>' {
                    let attr = &text[start..i];
                    finish_tag(&mut result, &mut attributes, false, attr, attrs)?;

                    start = i + 1;
                    context = Context::Text;
                }
            }
            Context::Close => {
                // When done a close tag.
                if c == '>' {
                    let attr = &text[start..i];
                    finish_tag(&mut result, &mut attributes, true, attr, attrs)?;

                    start = i + 1;
                    context = Context::Text;
                }
            }
        }
    }

    // Push remaining text into the result.

    if let Context::Open = context {
        start -= 1;
    }

    result.push_str(&text[start..]);
    for (attr, positions) in &attributes {
        if positions.len() % 2 != 0 {
            return Err(error!("Unmatched tag <{}>", attr));
        }
    }
    Ok((attributes, result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Map;

    #[test]
    fn test_extract_attr() {
        let attrs: Map<String, Option<String>> = hashmap! {
            "attr1".to_string() => None,
            "attr2".to_string() => None
        };

        let tests: Vec<(&str, Result<(Attributes, String)>)> = vec![
            (
                "Test <attr1>text</attr1>.",
                Ok((
                    hashmap! {
                        "attr1".to_string() => vec![5 as usize, 9]
                    },
                    "Test text.".to_string(),
                )),
            ),
            (
                "Test <b>text</b>.",
                Ok((Attributes::new(), "Test <b>text</b>.".to_string())),
            ),
            (
                "Test </attr1>text</attr1>.",
                Err(Error::Generic("Invalid closing tag </attr1>".to_string())),
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
            let result = extract_attr(text, &attrs);
            assert_eq!(result, expected);
        }
    }
}
