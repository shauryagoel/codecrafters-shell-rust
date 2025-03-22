#[allow(unused_imports)]
use std::io::{self, Write};

use std::process::ExitCode;

fn main() -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut input = String::new();
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap(); // Wait for user input

        // Trim removes the \n when pressing enter to run the command
        match input.trim() {
            "exit 0" => return ExitCode::from(0), // Return exit code from the program and exit it
            _ => {
                println!("{}: command not found", input.trim());
            }
        }

        input.clear(); // Clear the input string so that it can be used again without re-declaring the variable
    }
}
