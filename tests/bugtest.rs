use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator};
#[macro_use]
extern crate linear_map;

/// Tests attribute parsing.
#[test]
fn test_attributes() {
    let story: Story = Story::load_yml("./tests/data/bugtest").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    // println!("{:#?}", bookmark.state);
    // println!("{:#?}", story.sections["global"].passages);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();
    // return;

    let tests = vec![
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Else".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Yep".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Done".to_string(),
                ..Dialogue::default()
            }),
        ),
    ];

    for (input, line) in &tests {
        assert_eq!(&runner.next(input).unwrap(), line);
    }
}
