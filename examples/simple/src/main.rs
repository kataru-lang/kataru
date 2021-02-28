use colored::*;
use kataru::*;
use regex::{Captures, Regex};
use std::{
    borrow::Cow,
    io::{stdin, stdout, Write},
};

#[macro_use]
extern crate lazy_static;

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

fn run_command(command: &str, _params: &Map<String, Value>) {
    match command {
        "clearScreen" => print!("{}[2J", 27 as char),
        _ => println!("{}", "Unknown command".red()),
    }
}

fn replace_tags_ansi(text: &str) -> String {
    lazy_static! {
        static ref TAGS_RE: Regex = Regex::new(r"<([:a-zA-Z0-9_\./]*)>").unwrap();
    }
    TAGS_RE
        .replace_all(&text, |cap: &Captures| {
            let tag = &cap[1];
            let code = match tag {
                "b" => "\x1b[1m",
                "/b" => "\x1b[0m",
                _ => "",
            };
            Cow::from(code.to_string())
        })
        .to_string()
}

fn handle_line(runner: &mut Runner, input: &mut String) -> bool {
    let line = runner.next(&input).unwrap();
    match line {
        Line::Dialogue(dialogue) => {
            match dialogue.name.as_str() {
                "Narrator" => print!("{}", replace_tags_ansi(&dialogue.text).italic()),
                _ => print!(
                    "{}: {}",
                    dialogue.name.bold().yellow(),
                    replace_tags_ansi(&dialogue.text)
                ),
            }
            await_key(input);
            true
        }
        Line::Choices(choices) => {
            println!();
            for (choice, _passage_name) in &choices.choices {
                println!("{}", choice.cyan());
            }
            print!("\n{}", "Enter your choice: ".bold().magenta());
            get_input(input);
            true
        }
        Line::Commands(cmds) => {
            for cmd in cmds {
                for (command, params) in cmd {
                    run_command(command, params);
                }
            }
            true
        }
        Line::Input(input_cmd) => {
            for (_var, prompt) in &input_cmd.input {
                print!("{}: ", prompt.bold().magenta());
                get_input(input);
            }
            true
        }
        Line::InvalidChoice => {
            print!(
                "{}",
                format!("Invalid choice '{}', try again: ", input).magenta()
            );
            get_input(input);
            true
        }
        Line::End => {
            println!("End of story.");
            false
        }
        _ => {
            println!("Invalid line encountered: {:?}", line);
            false
        }
    }
}

fn main() {
    // Load the story.
    println!("{}", "Loading story...".bold().cyan());
    let mut bookmark = Bookmark::from_mp(include_bytes!("../target/bookmark")).unwrap();
    let story = Story::from_mp(include_bytes!("../target/story")).unwrap();

    #[cfg(debug_assertions)]
    print_validation(&story);

    let mut runner = Runner::new(&mut bookmark, &story).unwrap();
    let mut input = String::new();

    while handle_line(&mut runner, &mut input) {}
}
