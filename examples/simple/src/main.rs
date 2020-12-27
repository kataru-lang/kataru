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
fn print_validation(config: &Config, story: &Story) {
    // Validate the story.
    println!("{}", "Validating story...".bold().cyan());
    let msg = match validate(&config, &story) {
        Err(e) => format!("{}", e).red(),
        Ok(_) => "Validated story successfully.".bold().green(),
    };
    println!("{}\n", msg);
}

fn main() {
    // Load the story.
    println!("{}", "Loading story...".bold().cyan());
    let story: Story = Story::parse(include_str!("../story/passages/start.yml")).unwrap();
    let mut config: Config = Config::parse(include_str!("../story/config.yml")).unwrap();

    #[cfg(debug_assertions)]
    print_validation(&config, &story);

    let mut runner = Runner::new(&mut config, &story);

    let mut input = String::new();
    loop {
        match runner.next(&input) {
            Some(line) => match &line {
                Line::Text(text) => {
                    println!("{}", text.italic());
                    await_key(&mut input);
                }
                Line::Dialogue(dialogue) => {
                    let (name, quote) = dialogue.iter().next().unwrap();
                    println!("{}: {}", name.bold().yellow(), quote);
                    await_key(&mut input);
                }
                Line::Choices(choices) => {
                    for (choice, _passage_name) in &choices.choices {
                        println!("{}", choice.cyan());
                    }
                    print!("{}", "Enter your choice: ".magenta());
                    get_input(&mut input);
                }
                Line::InvalidChoice => {
                    print!(
                        "{}",
                        format!("Invalid choice '{}', try again: ", input).magenta()
                    );
                    get_input(&mut input);
                }
                _ => (),
            },
            None => break,
        }
    }
}
