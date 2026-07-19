use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator};

/// Tests calling commands from other namespaces
#[test]
fn test_read_line() {
    let story: Story = Story::load_yml("./tests/data/minimal").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    // println!("{:#?}", bookmark.state);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner = Runner::init(bookmark, story, true).unwrap();

    assert_eq!(
        runner.read_line().unwrap(),
        Line::Dialogue(Dialogue {
            name: "Alice".to_string(),
            text: "Hello!".to_string(),
            ..Dialogue::default()
        }),
    );
    println!("Line: {}", runner.bookmark().line());
    assert_eq!(
        runner.next("").unwrap(),
        Line::Dialogue(Dialogue {
            name: "Alice".to_string(),
            text: "Hello!".to_string(),
            ..Dialogue::default()
        }),
    );
    println!("Line: {}", runner.bookmark().line());
    assert_eq!(
        runner.next("").unwrap(),
        Line::Dialogue(Dialogue {
            name: "Bob".to_string(),
            text: "Nice to meet you!".to_string(),
            ..Dialogue::default()
        }),
    );
    println!("Line: {}", runner.bookmark().line());
    assert_eq!(
        runner.read_line().unwrap(),
        Line::Dialogue(Dialogue {
            name: "Alice".to_string(),
            text: "I should say this.".to_string(),
            ..Dialogue::default()
        }),
    );
    println!("Line: {}", runner.bookmark().line());
}
