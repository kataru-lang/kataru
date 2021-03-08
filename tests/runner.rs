use kataru::{Bookmark, Line, LoadYaml, Runner, Story, Validator, Value};
use maplit::btreemap;

#[test]
fn test_runner() {
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    let story: Story = Story::load_yml("./tests/data/story").unwrap();
    Validator::new(&story).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Commands(vec![
                btreemap! {"Alice.Wave".to_string() => btreemap! {"amount".to_string() => Value::Number(1.)}}
            ])
        );
    }

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Commands(vec![
                btreemap! {"Alice.Wave".to_string() => btreemap! {"amount".to_string() => Value::Number(0.)}}
            ])
        );
    }
}
