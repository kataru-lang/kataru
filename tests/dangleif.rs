use kataru::{Bookmark, Choices, Dialogue, Line, LoadYaml, Runner, Story, Validator};
use maplit::hashmap;

/// Tests basic $character commands.
#[test]
fn test_dangleif() {
    let story: Story = Story::load_yml("./tests/data/dangleif").unwrap();
    // println!("story: {:#?}", story);
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    let tests = vec![
        (
            "",
            Line::Choices(Choices {
                choices: hashmap! {
                    "Yes!".to_string() => "ChoiceYes".to_string(),
                    "No!".to_string() => "ChoiceNo".to_string()
                },
                timeout: 0.,
            }),
        ),
        (
            "Yes!",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Yes!".to_string(),
                attributes: hashmap! {},
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Success!".to_string(),
                attributes: hashmap! {},
            }),
        ),
    ];

    for (input, line) in &tests {
        assert_eq!(runner.next(input).unwrap(), line);
    }
}
