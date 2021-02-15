use kataru::{Load, Story, Validator};

#[test]
fn test_validate() {
    let story: Story = Story::load("./examples/simple/kataru/story").unwrap();
    Validator::new(&story).validate().unwrap();
}
