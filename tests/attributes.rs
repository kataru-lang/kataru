use kataru::{AttributedSpan, Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Value};
extern crate linear_map;
use maplit::hashmap;

/// Tests attribute parsing.
#[test]
fn test_attributes() {
    let story = Story::load_yml("./tests/data/attributes").unwrap();
    let bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    let mut runner = Runner::init(bookmark, story, true).unwrap();

    let tests = vec![
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Hello player!".to_string(),
                attributes: vec![AttributedSpan {
                    start: 6,
                    end: 12,
                    params: hashmap! { "wave".to_string() => Some(Value::Number(10.)) },
                }],
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "... hey!".to_string(),
                attributes: vec![AttributedSpan {
                    start: 4,
                    end: 4,
                    params: hashmap! { "sfx".to_string() => Some(Value::String("hey".to_string())),
                    "emote".to_string() => Some(Value::String("angry".to_string())),
                    "volume".to_string() => Some(Value::Number(10.))},
                }],
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "... hey again!".to_string(),
                attributes: vec![AttributedSpan {
                    start: 4,
                    end: 4,
                    params: hashmap! { "sfx".to_string() => Some(Value::String("hey".to_string())),
                    "emote".to_string() => Some(Value::String("angry".to_string())),
                    "volume".to_string() => Some(Value::Number(10.))},
                }],
            }),
        ),
    ];

    for (input, line) in &tests {
        assert_eq!(runner.next(input).unwrap(), line.clone());
    }
}
