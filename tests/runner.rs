use kataru::*;

#[test]
fn test_branch() {
    let config_str = "\
state:
    strength: 0
line: 0
passage: Test
characters:
    Alice:
        description: Alice.
cmds: {}
";
    let story_str = "\
Test:
    - if strength > 2:
        - Done
        - goto: End
      else:
        - But you weren't strong enough!
        - Should you try again?
        - choices:
            try again: Test
            train: End
End:
    - End.
";

    let mut config: Config = serde_yaml::from_str(&config_str).unwrap();
    let story: Story = serde_yaml::from_str(&story_str).unwrap();

    let mut runner = Runner::new(&mut config, &story);
    assert_eq!(
        runner.next(""),
        Some(Line::Text("But you weren't strong enough!".to_string()))
    );
}
