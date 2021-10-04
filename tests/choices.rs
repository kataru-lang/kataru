use kataru::{Bookmark, Choices, Dialogue, Line, LoadYaml, Runner, Story, Validator};

/// Tests basic $character commands.
#[test]
fn test_choices() {
    let story: Story = Story::load_yml("./tests/data/choices").unwrap();
    // println!("story: {:#?}", story);
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

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
