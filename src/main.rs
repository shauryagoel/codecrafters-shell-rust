#[allow(unused_imports)]
use std::io::{self, Write};

use std::env;
use std::fs;
use std::process::ExitCode;

// Check  if a command exists in the PATH environment variable
fn get_path(command_name: &str) -> Option<String> {
    let path_directories = env::var("PATH").unwrap();
    for directory in path_directories.split(":") {
        for file in fs::read_dir(directory).unwrap() {
            let file_name = file.as_ref().unwrap().file_name().into_string().unwrap();
            if file_name == command_name {
                return Some(file.as_ref().unwrap().path().into_os_string().into_string().unwrap());
            }
        }
    }
    None
}

fn main() -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let builtin_commands = ["exit", "echo", "type"];

    let mut input = String::new();
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap(); // Wait for user input
        let trimmed_input = input.trim(); // Trim also removes the \n when pressing enter to run the command

        // Split at the first space character
        // to form the format of- `command arg1 arg2 ...`
        match trimmed_input.split_once(" ") {
            Some(x) => match x.0 {
                "exit" => return ExitCode::from(x.1.as_bytes()[0] - 48), // Return exit code from the program and exit it
                "echo" => println!("{}", x.1),
                "type" => {
                    if builtin_commands.contains(&x.1) {
                        println!("{} is a shell builtin", x.1);
                    } else if let Some(path) = get_path(x.1) {
                        println!("{} is {}", x.1, path)
                    } else {
                        println!("{}: not found", x.1);
                    }
                }
                _ => println!("{} {}: command not found", x.0, x.1),
            },
            None => println!("{}: command not found", trimmed_input),
        }
        input.clear(); // Clear the input string so that it can be used again without re-declaring the variable
    }
}
