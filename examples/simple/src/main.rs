use colored::*;
use kataru::*;
use std::io::{stdin, stdout, Write};

fn get_input(input: &mut String) {
    let _ = stdout().flush();
    *input = String::new();
    stdin().read_line(input).expect("Invalid input");
    loop {
        if let Some('\n') = input.chars().next_back() {
            input.pop();
        } else if let Some('\r') = input.chars().next_back() {
            input.pop();
        } else {
            break;
        }
    }
}

fn await_key(input: &mut String) {
    get_input(input);
    *input = String::new();
}

#[cfg(debug_assertions)]
fn print_validation(story: &Story) {
    // Validate the story.
    println!("{}", "Validating story...".bold().cyan());
    let msg = match Validator::new(story).validate() {
        Err(e) => format!("{}", e).red(),
        Ok(_) => "Validated story successfully.".bold().green(),
    };
    println!("{}\n", msg);
}

fn main() {
    // Load the story.
    println!("{}", "Loading story...".bold().cyan());
    let mut bookmark = Bookmark::deserialize(include_bytes!("../target/bookmark"));
    let story = Story::deserialize(include_bytes!("../target/story"));

    #[cfg(debug_assertions)]
    print_validation(&story);

    let mut runner = Runner::new(&mut bookmark, &story);

    let mut input = String::new();
    loop {
        match runner.next(&input) {
            Line::Text(text) => {
                print!("{}", text.italic());
                await_key(&mut input);
            }
            Line::Dialogue(dialogue) => {
                let (name, quote) = dialogue.iter().next().unwrap();
                print!("{}: {}", name.bold().yellow(), quote);
                await_key(&mut input);
            }
            Line::Choices(choices) => {
                println!();
                for (choice, _passage_name) in &choices.choices {
                    println!("{}", choice.cyan());
                }
                print!("\n{}", "Enter your choice: ".magenta());
                get_input(&mut input);
            }
            Line::Cmd(cmd) => match cmd.cmd.as_str() {
                "clearScreen" => print!("{}[2J", 27 as char),
                _ => (),
            },
            Line::InvalidChoice => {
                print!(
                    "{}",
                    format!("Invalid choice '{}', try again: ", input).magenta()
                );
                get_input(&mut input);
            }
            Line::Error | Line::End => {
                break;
            }
            _ => (),
        }
    }
}
