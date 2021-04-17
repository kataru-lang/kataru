use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator, Value};
use maplit::btreemap;
#[macro_use]
extern crate linear_map;

/// Tests calling commands from other namespaces
#[test]
fn test_namespaces() {
    let story: Story = Story::load_yml("./tests/data/namespaces").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    println!("{:#?}", story);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    let tests = vec![
        (
            "",
            Line::Dialogue(Dialogue {
                name: "GlobalCharacter".to_string(),
                text: "Hello".to_string(),
                attributes: btreemap! {},
            }),
        ),
        (
            "",
            Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("".to_string())}},
            ),
        ),
        (
            "",
            Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("test".to_string())}},
            ),
        ),
        // Test only string param
        (
            "",
            Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("test".to_string())}},
            ),
        ),
        (
            "",
            Line::Command(
                btreemap! {"GlobalCommand".to_string() => linear_map! {"param".to_string() => Value::Number(0.)}},
            ),
        ),
        (
            "",
            Line::Command(
                btreemap! {"GlobalCommand".to_string() => linear_map! {"param".to_string() => Value::Number(1.)}},
            ),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "namespace1:LocalCharacter".to_string(),
                text: "Hello".to_string(),
                attributes: btreemap! {},
            }),
        ),
        (
            "",
            Line::Dialogue(Dialogue {
                name: "GlobalCharacter".to_string(),
                text: "Hello".to_string(),
                attributes: btreemap! {},
            }),
        ),
        (
            "",
            Line::Command(btreemap! {
                "namespace1:LocalCharacter.LocalMethod".to_string() => linear_map! {
                    "param1".to_string() => Value::Number(1.),
                    "param2".to_string() => Value::String("two".to_string()),
                    "param3".to_string() => Value::Bool(true)
                }
            }),
        ),
        (
            "",
            Line::Command(btreemap! {
                "namespace1:LocalCharacter.LocalMethod".to_string() => linear_map! {
                    "param1".to_string() => Value::Number(1.),
                    "param2".to_string() => Value::String("two".to_string()),
                    "param3".to_string() => Value::Bool(true)
                }
            }),
        ),
        (
            "",
            Line::Command(btreemap! {
                "namespace1:LocalCharacter.LocalMethod".to_string() => linear_map! {
                    "param1".to_string() => Value::Number(1.),
                    "param2".to_string() => Value::String("two".to_string()),
                    "param3".to_string() => Value::Bool(false)
                }
            }),
        ),
        (
            "",
            Line::Command(
                btreemap! {"namespace1:LocalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("".to_string())}},
            ),
        ),
        // Make sure global characters don't get the namespace appended
        (
            "",
            Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("".to_string())}},
            ),
        ),
    ];

    for (input, line) in &tests {
        assert_eq!(runner.next(input).unwrap(), line);
    }
}
