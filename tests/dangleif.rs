use kataru::{Bookmark, LoadYaml, Runner, Story, Validator};

/// Tests basic $character commands.
#[test]
fn test_dangleif() {
    let story: Story = Story::load_yml("./tests/data/dangleif").unwrap();
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    Validator::new(&story, &bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    // Alice: Test
    runner.next("").unwrap();
}
