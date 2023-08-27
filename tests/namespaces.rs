use kataru::{Bookmark, Command, Dialogue, Line, LoadYaml, Runner, Story, Validator, Value};
#[macro_use]
extern crate linear_map;

/// Tests calling commands from other namespaces
#[test]
fn test_namespaces() {
    let story: Story = Story::load_yml("./tests/data/namespaces").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    // println!("{:#?}", bookmark.state);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner = Runner::new(bookmark, story, true).unwrap();

    let tests = vec![
        (
            "",
            Line::Dialogue(Dialogue {
                name: "GlobalCharacter".to_string(),
                text: "Hello".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "GlobalCharacter.GlobalMethod".to_string(),
                params: linear_map! {"param".to_string() => Value::String("".to_string())},
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "GlobalCharacter.GlobalMethod".to_string(),
                params: linear_map! {"param".to_string() => Value::String("test".to_string())},
            }),
        ),
        // Test only string param
        (
            "",
            Line::Command(Command {
                name: "GlobalCharacter.GlobalMethod".to_string(),
                params: linear_map! {"param".to_string() => Value::String("test".to_string())},
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "GlobalCommand".to_string(),
                params: linear_map! {"param".to_string() => Value::Number(0.)},
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "GlobalCommand".to_string(),
                params: linear_map! {"param".to_string() => Value::Number(1.)},
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "GlobalCommandNoParams".to_string(),
                params: linear_map! {},
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "namespace1:LocalCharacter".to_string(),
                text: "Hello".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "GlobalCharacter".to_string(),
                text: "Hello".to_string(),
                ..Dialogue::default()
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "namespace1:LocalCharacter.LocalCharacterMethod".to_string(),
                params: linear_map! {"param1".to_string() => Value::Number(1.),
                "param2".to_string() => Value::String("two".to_string()),
                "param3".to_string() => Value::Bool(true)},
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "namespace1:LocalCharacter.LocalCharacterMethod".to_string(),
                params: linear_map! {"param1".to_string() => Value::Number(3.),
                "param2".to_string() => Value::String("two".to_string()),
                "param3".to_string() => Value::Bool(true)},
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "namespace1:LocalCharacter.LocalCharacterMethod".to_string(),
                params: linear_map! {"param1".to_string() => Value::Number(1.),
                "param2".to_string() => Value::String("two".to_string()),
                "param3".to_string() => Value::Bool(false)},
            }),
        ),
        (
            "",
            Line::Command(Command {
                name: "namespace1:LocalCharacter.GlobalMethod".to_string(),
                params: linear_map! {
                "param".to_string() => Value::String("".to_string())},
            }),
        ),
        // Make sure global characters don't get the namespace appended
        (
            "",
            Line::Command(Command {
                name: "GlobalCharacter.GlobalMethod".to_string(),
                params: linear_map! {
                "param".to_string() => Value::String("".to_string())},
            }),
        ),
        // Make sure local commands get namespace appended.
        (
            "",
            Line::Command(Command {
                name: "namespace1:LocalCommand".to_string(),
                params: linear_map! {
                "param".to_string() => Value::Number(0.0)},
            }),
        ),
        // - LocalCharacter: Visited namespace2 start $Start:visited time(s).
        (
            "",
            Line::Dialogue(Dialogue {
                name: "namespace1:LocalCharacter".to_string(),
                text: "Visited namespace2 start 1 time(s)".to_string(),
                ..Dialogue::default()
            }),
        ),
        // - LocalCharacter: Value of namespace1:var is $var
        (
            "",
            Line::Dialogue(Dialogue {
                name: "namespace1:LocalCharacter".to_string(),
                text: "Value of namespace1:var is false".to_string(),
                ..Dialogue::default()
            }),
        ),
         // Make sure local commands work in other namespaces.
         (
            "",
            Line::Command(Command {
                name: "namespace1:LocalCommand".to_string(),
                params: linear_map! {
                "param".to_string() => Value::Number(0.0)},
            }),
        ),
    ];

    for (input, line) in &tests {
        assert_eq!(&runner.next(input).unwrap(), line);
    }
}
