use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story};

/// Tests loading commented out story files and config-only story files.
#[test]
fn test_story2() {
    let story: Story = Story::load_yml("./tests/data/file_formats").unwrap();
    let bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    let mut runner = Runner::init(bookmark, story, true).unwrap();

    let tests = vec![(
        "",
        Line::Dialogue(Dialogue {
            name: "Alice".to_string(),
            text: "Test story!".to_string(),
            ..Dialogue::default()
        }),
    )];

    for (input, line) in &tests {
        assert_eq!(&runner.next(input).unwrap(), line);
    }
}

/// Tests loading commented out story files and config-only story files.
#[test]
fn test_default_bookmark() {
    let story: Story = Story::load_yml("./tests/data/file_formats").unwrap();
    let bookmark: Bookmark = Bookmark::load_or_default(
        "./tests/data/missing-bookmark.yml",
        &story,
        "Start".to_string(),
    )
    .unwrap();
    let mut runner = Runner::init(bookmark, story, true).unwrap();

    let tests = vec![(
        "",
        Line::Dialogue(Dialogue {
            name: "Alice".to_string(),
            text: "Test story!".to_string(),
            ..Dialogue::default()
        }),
    )];

    for (input, line) in &tests {
        assert_eq!(&runner.next(input).unwrap(), line);
    }
}
