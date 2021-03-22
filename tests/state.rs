use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator, Value};
use maplit::btreemap;
#[macro_use]
extern crate linear_map;

/// Tests basic $character commands.
#[test]
fn test_state() {
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    let story: Story = Story::load_yml("./tests/data/state").unwrap();
    Validator::new(&story).validate().unwrap();

    bookmark.init_state(&story);
    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

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

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(1.)}}
            )
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(2.)}}
            )
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Command(
                btreemap! {"Alice.Wave".to_string() => linear_map! {"amount".to_string() => Value::Number(0.)}}
            )
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "0 leq 0".to_string(),
                attributes: btreemap! {}
            })
        );
    }
}
