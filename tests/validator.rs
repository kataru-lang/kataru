use kataru::*;

#[test]
fn test_validate() {
    let config_str = include_str!("../examples/simple/story/config.yml");
    let story_str = include_str!("../examples/simple/story/passages/start.yml");
    let config: Config = serde_yaml::from_str(&config_str).unwrap();
    let story: Story = serde_yaml::from_str(&story_str).unwrap();
    validate(&config, &story).unwrap();
}
