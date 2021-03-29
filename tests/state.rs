use kataru::{Bookmark, Choices, Dialogue, Line, LoadYaml, Runner, Story, Validator, Value};
use maplit::btreemap;
#[macro_use]
extern crate linear_map;

/// Tests basic $character commands.
#[test]
fn test_state() {
    let story: Story = Story::load_yml("./tests/data/state").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    println!("{:#?}", bookmark);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    // Alice.Wave {amount: 1}
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"TestBool".to_string() => linear_map! {"bool".to_string() => Value::Bool(false)}}
            )
        );
    }

    // Alice: Test
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Test".to_string(),
                attributes: btreemap! {}
            })
        );
    }

    // Alice.Wave {amount: 1}
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(1.)}}
            )
        );
    }

    // Alice.Wave {amount: 2}
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(2.)}}
            )
        );
    }

    // Alice.Wave {amount: 0}
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(0.)}}
            )
        );
    }

    // Alice: 0 neq 0
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "0 neq 0".to_string(),
                attributes: btreemap! {}
            })
        );
    }

    // Alice: 0 geq 0
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Choices(Choices {
                choices: btreemap! {
                    "Choice1".to_string() => "Choice1".to_string(),
                    "Choice2".to_string() => "Choice2".to_string()
                },
                timeout: 0.
            })
        );
    }

    // Alice: Choice1
    {
        let line = runner.next("Choice1").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Choice1".to_string(),
                attributes: btreemap! {}
            })
        );
    }

    // Set var = 4
    // Check if > 3 and < 5
    // 3 < var < 5
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "3 < var < 5".to_string(),
                attributes: btreemap! {}
            })
        );
    }

    // Alice: End
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "End".to_string(),
                attributes: btreemap! {}
            })
        );
    }
}
