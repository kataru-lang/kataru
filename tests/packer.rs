use kataru::{pack, FromMessagePack, Story};
use std::fs;

#[test]
fn test_pack() {
    pack("./examples/simple/kataru", "./target").unwrap();
    let _story = Story::from_mp(&fs::read("./target/story").unwrap());
}
