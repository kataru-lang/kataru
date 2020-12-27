use kataru::*;

#[test]
fn test_validate() {
    let config: Config =
        Config::parse(include_str!("../examples/simple/story/config.yml")).unwrap();
    let story: Story =
        Story::parse(include_str!("../examples/simple/story/passages/start.yml")).unwrap();
    validate(&config, &story).unwrap();
}
