use kataru::{Bookmark, LoadYaml, Runner, Story, Validator};

/// Tests basic $character commands.
#[test]
fn test_dangleif() {
    let story: Story = Story::load_yml("./tests/data/dangleif").unwrap();
    // println!("story: {:#?}", story);
    let mut bookmark: Bookmark = Bookmark::load_yml("./tests/data/bookmark.yml").unwrap();
    bookmark.init_state(&story);

    Validator::new(&story, &mut bookmark).validate().unwrap();

    let mut runner: Runner = Runner::new(&mut bookmark, &story).unwrap();

    runner.next("").unwrap();
}
