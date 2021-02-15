use kataru::{LoadYaml, Story, Validator};

#[test]
fn test_validate() {
    let story: Story = Story::load_yml("./examples/simple/kataru/story").unwrap();
    Validator::new(&story).validate().unwrap();
}
