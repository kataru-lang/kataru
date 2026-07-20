use std::collections::HashMap;

use kataru::{
    Bookmark, ChoiceTarget, Choices, Config, Dialogue, GLOBAL, Line, LoadYaml, Map, Passage,
    Passages, Position, RawChoices, RawLine, Runner, Section, Story,
};

/// Tests basic $character commands.
#[test]
fn test_choices() {
    let story = Story::load_yml("./tests/data/choices").unwrap();
    let bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    let mut runner = Runner::init(bookmark, story, true).unwrap();

    let tests = vec![
        (
            "",
            Line::Choices(Choices {
                choices: vec!["yes".to_string(), "no".to_string()],
                ..Choices::default()
            }),
        ),
        (
            "yes",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Yes!".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Choices(Choices {
                choices: vec!["yes".to_string(), "no".to_string(), "maybe".to_string()],
                ..Choices::default()
            }),
        ),
        (
            "yes",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Embedded yes 1".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Choices(Choices {
                choices: vec!["yes".to_string(), "no".to_string()],
                ..Choices::default()
            }),
        ),
        (
            "yes",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "yes".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Default".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Embedded default".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "var1 > 0".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Success!".to_string(),
                ..Dialogue::default()
            }),
        ),
    ];

    for (input, line) in &tests {
        let real_line = runner.next(input).unwrap();
        assert_eq!(&real_line, line);
    }
}

#[test]
fn test_default() {
    let story = Story {
        sections: HashMap::from([(
            GLOBAL.into(),
            Section {
                config: Config {
                    characters: HashMap::from([("Alice".into(), None)]),
                    ..Default::default()
                },
                passages: Passages::from([
                    (
                        "Start".into(),
                        Passage::from([RawLine::Choices(RawChoices {
                            default: ChoiceTarget::PassageName("Default".into()),
                            ..Default::default()
                        })]),
                    ),
                    (
                        "Default".into(),
                        Passage::from([RawLine::Dialogue(Map::from([(
                            "Alice".into(),
                            "Default".into(),
                        )]))]),
                    ),
                ]),
            },
        )]),
    };
    let bookmark = Bookmark {
        position: Position {
            passage: "Start".into(),
            namespace: GLOBAL.into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut runner = Runner::init(bookmark, story, true).unwrap();
    assert_eq!(
        runner.next(""),
        Ok(Line::Dialogue(Dialogue {
            name: "Alice".to_string(),
            text: "Default".to_string(),
            ..Dialogue::default()
        })),
    )
}
