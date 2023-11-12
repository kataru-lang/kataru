use kataru::{
    AssignOperator, Bookmark, Choices, Command, Dialogue, Input, Line, Load, Runner, Save,
    StateMod, Story, Value,
};
use maplit::hashmap;
#[macro_use]
extern crate linear_map;

/// Tests basic $character commands.
#[test]
fn test_state() {
    // Load story from directory.
    let story: Story = Story::load("./tests/data/state").unwrap();
    let bookmark: Bookmark = Bookmark::load("./tests/data/bookmark.yml").unwrap();
    let mut runner = Runner::init(bookmark, story, true).unwrap();

    runner
        .set_state(
            StateMod {
                var: "var",
                op: AssignOperator::None,
            },
            Value::Number(2.0),
        )
        .unwrap();
    assert_eq!(runner.bookmark().value("var").unwrap(), &Value::Number(2.0));
    runner
        .set_state(
            StateMod {
                var: "var",
                op: AssignOperator::None,
            },
            Value::Number(1.0),
        )
        .unwrap();
    assert_eq!(runner.bookmark().value("var").unwrap(), &Value::Number(1.0));

    let tests = vec![
        // TestBool: { bool: not $boolVar }
        (
            "",
            Line::Command(Command {
                name: "TestBool".to_string(),
                params: linear_map! {"bool".to_string() => Value::Bool(false)},
            }),
        ),
        // Alice: Test
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Test".to_string(),
                ..Dialogue::default()
            }),
        ),
        // Alice.Wave: { amount: $var } # $var = 1
        (
            "",
            Line::Command(Command {
                name: "Alice.Wave".to_string(),
                params: linear_map! {"amount".to_string() =>Value::Number(1.)},
            }),
        ),
        // Alice.Wave: { amount: $var } # $var = 2
        (
            "",
            Line::Command(Command {
                name: "Alice.Wave".to_string(),
                params: linear_map! {"amount".to_string() =>Value::Number(2.)},
            }),
        ),
        // Alice.Wave: { amount: $var } # $var = 2
        (
            "",
            Line::Command(Command {
                name: "Alice.Wave".to_string(),
                params: linear_map! {"amount".to_string() =>Value::Number(0.)},
            }),
        ),
        // Alice: $var neq 0
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "0 neq 0".to_string(),
                ..Dialogue::default()
            }),
        ),
        // choices:
        //   choice1 text: Choice1
        //   choice2 text: Choice2
        (
            "",
            Line::Choices(Choices {
                choices: vec!["choice1 text".to_string(), "choice2 text".to_string()],
                ..Choices::default()
            }),
        ),
        // Alice: Choice1
        (
            "choice1 text",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Choice1".to_string(),
                ..Dialogue::default()
            }),
        ),
        // input:
        //   $name: What's your name?
        (
            "",
            Line::Input(Input {
                timeout: 0.0,
                input: hashmap! {
                    "$name".to_string() => "What's your name?".to_string()
                },
            }),
        ),
        // var > $THREE
        (
            "Player",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "var > 3".to_string(),
                ..Dialogue::default()
            }),
        ),
        // 3 < var < 5
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "3 < var < 5".to_string(),
                ..Dialogue::default()
            }),
        ),
        // Alice: Alice: Visited Choice1 $Choice1.visited times.
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Visited Choice1 1 times.".to_string(),
                ..Dialogue::default()
            }),
        ),
        // Alice: Alice: Exited Choice1 $Choice1.exited times.
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Exited Choice1 1 times.".to_string(),
                ..Dialogue::default()
            }),
        ),
        // Alice: Alice: Exited Choice1Intermediate $Choice1Intermediate.exited times.
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Exited Choice1Intermediate 1 times.".to_string(),
                ..Dialogue::default()
            }),
        ),
        // 3 + 4 = {$THREE + $FOUR}
        (
            "",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "3 + 4 = 7".to_string(),
                ..Dialogue::default()
            }),
        ),
        // - choices:
        //     if $Choice1.exited > 0: { choice1 text: Choice1 }
        //     if $Choice2.exited > 0: { choice2 text: Choice2 }
        (
            "",
            Line::Choices(Choices {
                choices: vec!["choice1 text".to_string()],
                ..Choices::default()
            }),
        ),
        // Alice: Choice1
        (
            "choice1 text",
            Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Choice1".to_string(),
                ..Dialogue::default()
            }),
        ),
    ];

    for (input, line) in &tests {
        assert_eq!(&runner.next(input).unwrap(), line);
    }

    // Try the same tests on the compiled.
    let compiled_story_path = "./tests/data/compiled/state/compiled_story.bin";
    Story::load("./tests/data/state")
        .unwrap()
        .save(compiled_story_path)
        .unwrap();
    let story = Story::load(compiled_story_path).unwrap();
    let bookmark: Bookmark = Bookmark::load("./tests/data/bookmark.yml").unwrap();
    runner = Runner::init(bookmark, story, true).unwrap();

    for (input, line) in &tests {
        assert_eq!(&runner.next(input).unwrap(), line);
    }

    // Try running just the default test.
    assert_eq!(
        runner.run("Default".to_string()).unwrap(),
        Line::Dialogue(Dialogue {
            name: "Alice".to_string(),
            text: "default".to_string(),
            ..Dialogue::default()
        })
    );
    // Make sure the stack was cleared and we don't return to some previous passage.
    assert_eq!(runner.next("").unwrap(), Line::End);
}
