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

fn run_command(command: &str, _params: &Map<String, Value>) {
    match command {
        "clearScreen" => print!("{}[2J", 27 as char),
        _ => println!("{}", "Unknown command".red()),
    }
}

fn handle_line(runner: &mut Runner, input: &mut String) -> bool {
    match runner.next(&input) {
        Line::Dialogue(dialogue) => {
            let (name, quote) = dialogue.iter().next().unwrap();
            match name.as_str() {
                "Narrator" => print!("{}", quote.italic()),
                _ => print!("{}: {}", name.bold().yellow(), quote),
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
        Line::Cmds(cmds) => {
            for cmd in cmds {
                for (command, params) in cmd {
                    run_command(command, params);
                }
            }
            true
        }
        Line::InputCmd(input_cmd) => {
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
        _ => false,
    }
}

fn main() {
    // Load the story.
    println!("{}", "Loading story...".bold().cyan());
    let mut bookmark = Bookmark::from_mp(include_bytes!("../target/bookmark"));
    let story = Story::from_mp(include_bytes!("../target/story"));

    #[cfg(debug_assertions)]
    print_validation(&story);

    let mut runner = Runner::new(&mut bookmark, &story);
    let mut input = String::new();

    while handle_line(&mut runner, &mut input) {}
}
