use kataru::*;

#[test]
fn test_autogen() {
    let story: Story =
        Story::load_yml(r"C:\Users\Joshi\Dev\Unity\JongelaMirrors\Assets\Kataru\Story").unwrap();
    for (namespace, section) in story {
        for (passage_name, _passage) in section.passages {
            println!("\"{}:{}\",", namespace, passage_name);
        }
    }
}
