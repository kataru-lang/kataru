use kataru::{Bookmark, Choices, Dialogue, Line, LoadYaml, Runner, Story, Validator, Value};
use maplit::hashmap;
#[macro_use]
extern crate linear_map;

/// Tests basic $character commands.
#[test]
fn test_state() {
    let story: Story = Story::load_yml("./tests/data/state").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    // println!("{:#?}", bookmark);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    let tests = vec![
        // TestBool: { bool: not $boolVar }
        (
            "",
            Line::Command(
                hashmap! {"TestBool".to_string() => linear_map! {"bool".to_string() => Value::Bool(false)}},
            ),
        ),
        // Alice: Test
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Test".to_string(),
                attributes: hashmap! {},
            }),
        ),
        // Alice.Wave: { amount: $var } # $var = 1
        (
            "",
            Line::Command(
                hashmap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(1.)}},
            ),
        ),
        // Alice.Wave: { amount: $var } # $var = 2
        (
            "",
            Line::Command(
                hashmap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(2.)}},
            ),
        ),
        // Alice.Wave: { amount: $var } # $var = 2
        (
            "",
            Line::Command(
                hashmap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(0.)}},
            ),
        ),
        // Alice: $var neq 0
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "0 neq 0".to_string(),
                attributes: hashmap! {},
            }),
        ),
        // choices:
        //   Choice1: Choice1
        //   Choice2: Choice2
        (
            "",
            Line::Choices(Choices {
                choices: hashmap! {
                    "Choice1".to_string() => "Choice1".to_string(),
                    "Choice2".to_string() => "Choice2".to_string()
                },
                timeout: 0.,
            }),
        ),
        // Alice: Choice1
        (
            "Choice1",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Choice1".to_string(),
                attributes: hashmap! {},
            }),
        ),
        // var > $THREE
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "var > 3".to_string(),
                attributes: hashmap! {},
            }),
        ),
        // 3 < var < 5
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "3 < var < 5".to_string(),
                attributes: hashmap! {},
            }),
        ),
        // Alice: Alice: Visited Choice1 $Choice1.visited times.
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Visited Choice1 1 times.".to_string(),
                attributes: hashmap! {},
            }),
        ),
        // Alice: Alice: Exited Choice1 $Choice1.exited times.
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Exited Choice1 1 times.".to_string(),
                attributes: hashmap! {},
            }),
        ),
        // Alice: Alice: Exited Choice1Intermediate $Choice1Intermediate.exited times.
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Exited Choice1Intermediate 1 times.".to_string(),
                attributes: hashmap! {},
            }),
        ),
        // Alice: End
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "End".to_string(),
                attributes: hashmap! {},
            }),
        ),
    ];

    for (input, line) in &tests {
        assert_eq!(runner.next(input).unwrap(), line);
    }
}
