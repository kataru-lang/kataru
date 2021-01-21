use kataru::{pack, Deserializable, Story};
use std::fs;

#[test]
fn test_pack() {
    pack("./examples/simple/kataru", "./target").unwrap();
    let _story = Story::deserialize(&fs::read("./target/story").unwrap());
}
