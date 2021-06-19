use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator};

/// Tests loading commented out story files and config-only story files.
#[test]
fn test_story2() {
    let story: Story = Story::load_yml("./tests/data/file_formats").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

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
