use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator, Value};
use maplit::btreemap;

/// Tests calling commands from other namespaces
#[test]
fn test_story3() {
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
            &Line::Commands(vec![
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => btreemap! {"param".to_string() => Value::Number(0.)}}
            ])
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Commands(vec![
                btreemap! {"GlobalCharacter.GlobalMethod".to_string() => btreemap! {"param".to_string() => Value::Number(1.)}}
            ])
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Commands(vec![
                btreemap! {"GlobalCommand".to_string() => btreemap! {"param".to_string() => Value::Number(0.)}}
            ])
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Commands(vec![
                btreemap! {"GlobalCommand".to_string() => btreemap! {"param".to_string() => Value::Number(1.)}}
            ])
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
            &Line::Commands(vec![
                btreemap! {"namespace1:LocalCharacter.LocalMethod".to_string() => btreemap! {"param".to_string() => Value::Number(0.)}}
            ])
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Commands(vec![
                btreemap! {"namespace1:LocalCharacter.GlobalMethod".to_string() => btreemap! {"param".to_string() => Value::Number(0.)}}
            ])
        );
    }
}
