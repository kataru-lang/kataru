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
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("".to_string())}}
            )
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("test".to_string())}}
            )
        );
    }

    // Test only a string param
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("test".to_string())}}
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
                btreemap! {"namespace1:LocalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("".to_string())}}
            )
        );
    }

    // Make sure global characters don't get the namespace appended
    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => linear_map! {"param".to_string() => Value::String("".to_string())}}
            )
        );
    }
}
