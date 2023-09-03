use kataru::{Bookmark, Choices, Dialogue, Line, LoadYaml, Runner, Story};

/// Tests basic $character commands.
#[test]
fn test_conditionals() {
    let story: Story = Story::load_yml("./tests/data/conditionals").unwrap();
    let bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    let mut runner: Runner = Runner::init(bookmark, story, true).unwrap();

    let tests = vec![
        (
            "",
            Line::Choices(Choices {
                choices: vec!["Yeah!".to_string(), "Yes!".to_string(), "No!".to_string()],
                ..Choices::default()
            }),
        ),
        (
            "Yes!",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Yes!".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "I will say this.".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "I will also say this.".to_string(),
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
