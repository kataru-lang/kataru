use kataru::{Bookmark, Dialogue, Line, LoadYaml, Runner, Story, Validator};
use maplit::btreemap;

/// Tests loading commented out story files and config-only story files.
#[test]
fn test_story2() {
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    let story: Story = Story::load_yml("./tests/data/file_formats").unwrap();
    Validator::new(&story).validate().unwrap();

    bookmark.init_state(&story);
    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    {
        let line = runner.next("").unwrap();
        assert_eq!(
            line,
            &Line::Dialogue(Dialogue {
                name: "Alice".to_string(),
                text: "Test story!".to_string(),
                attributes: btreemap! {}
            })
        );
    }
}
