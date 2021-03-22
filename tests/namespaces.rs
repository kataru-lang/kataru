use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator, Value};
use maplit::btreemap;
#[macro_use]
extern crate linear_map;

/// Tests calling commands from other namespaces
#[test]
fn test_namespaces() {
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    let story: Story = Story::load_yml("./tests/data/namespaces").unwrap();

    println!("{:#?}", story);
    Validator::new(&story).validate().unwrap();

    bookmark.init_state(&story);
    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "GlobalCharacter".to_string(),
                text: "Hello".to_string(),
                attributes: btreemap! {}
            })
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::Number(0.)}}
            )
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::Number(1.)}}
            )
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"GlobalCommand".to_string() => linear_map! {"param".to_string() => Value::Number(0.)}}
            )
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"GlobalCommand".to_string() => linear_map! {"param".to_string() => Value::Number(1.)}}
            )
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "namespace1:LocalCharacter".to_string(),
                text: "Hello".to_string(),
                attributes: btreemap! {}
            })
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(btreemap! {
                "namespace1:LocalCharacter.LocalMethod".to_string() => linear_map! {
                    "param1".to_string() => Value::Number(1.),
                    "param2".to_string() => Value::String("two".to_string()),
                    "param3".to_string() => Value::Bool(true)
                }
            })
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(btreemap! {
                "namespace1:LocalCharacter.LocalMethod".to_string() => linear_map! {
                    "param1".to_string() => Value::Number(1.),
                    "param2".to_string() => Value::String("two".to_string()),
                    "param3".to_string() => Value::Bool(false)
                }
            })
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"namespace1:LocalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::Number(0.)}}
            )
        );
    }
}
