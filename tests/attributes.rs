use kataru::{AttributedSpan, Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator, Value};
#[macro_use]
extern crate linear_map;
use maplit::hashmap;

/// Tests attribute parsing.
#[test]
fn test_attributes() {
    let story: Story = Story::load_yml("./tests/data/attributes").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    // println!("{:#?}", bookmark.state);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

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
        assert_eq!(&runner.next(input).unwrap(), line);
    }
}
