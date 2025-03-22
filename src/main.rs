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
        let trimmed_input = input.trim(); // Trim also removes the \n when pressing enter to run the command

        match trimmed_input.split_once(" ") {
            Some(x) => match x.0 {
                "exit" => return ExitCode::from(x.1.as_bytes()[0] - 48), // Return exit code from the program and exit it
                "echo" => println!("{}", x.1),
                _ => println!("{} {}: command not found", x.0, x.1),
            },
            None => println!("{}: command not found", trimmed_input),
        }
        input.clear(); // Clear the input string so that it can be used again without re-declaring the variable
    }
}
