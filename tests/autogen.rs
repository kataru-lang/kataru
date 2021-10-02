use kataru::*;

#[test]
fn test_autogen() {
    let story: Story = Story::load_yml("./tests/data/namespaces").unwrap();
    for (namespace, section) in story.sections {
        for (passage_name, _passage) in section.passages {
            println!("\"{}:{}\",", namespace, passage_name);
        }
    }
}
