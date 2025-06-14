use std::io::{self, Write};

use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

// Parse error code from the Optional str (will be None if the exit command was not given any exit code)
// Set default of 0 status code if received no status code
// If unable to parse the string of status code to u8 then give 1 status code
fn get_exit_code(str_code: Option<&&str>) -> u8 {
    str_code.unwrap_or(&"0").parse::<u8>().unwrap_or(1)
}

// Return absolute path of the command if found in the PATH environment variable
fn get_absolute_command_path(command_name: &str) -> Option<String> {
    let path_directories = env::var("PATH").unwrap();
    for directory in path_directories.split(":") {
        for file in fs::read_dir(directory).unwrap() {
            let file_name = file.as_ref().unwrap().file_name().into_string().unwrap();
            if file_name == command_name {
                return Some(file.unwrap().path().to_str().unwrap().to_owned());
            }
        }
    }
    None
}

// Print the vector of strings to stdout
// This function assumes that the `parsed_args` is already in a clean state
// like removing duplicate spaces, handling quotes, etc.
fn builtin_echo(parsed_args: Vec<&str>) {
    for arg in parsed_args {
        print!("{arg}")
    }
    println!();
}

fn builtin_cd(path: Option<&&str>) {
    // TODO: Maybe try `Cow` to avoid using String
    let corrected_path = if path.unwrap_or(&"~") == &"~" {
        env::var("HOME").unwrap()
    } else {
        (*path.unwrap()).to_owned()
    };

    let path_obj = Path::new(corrected_path.as_str());

    // Changes the current directory
    // if error occurs like no directory exists, then, print an error
    if env::set_current_dir(path_obj).is_err() {
        println!("cd: {}: No such file or directory", corrected_path);
    }
}

// Implement the type command
fn builtin_type(command: &str) {
    // Command implemented in the current program are called builtin commands
    // NOTE: If adding new builtin command, make sure to add it below
    // TODO: use enums for this to reduce human errors
    let builtin_commands = ["exit", "echo", "type", "pwd", "cd"];

    if builtin_commands.contains(&command) {
        println!("{} is a shell builtin", command);
    } else if let Some(command_path) = get_absolute_command_path(command) {
        println!("{} is {}", command, command_path)
    } else {
        println!("{}: not found", command);
    }
}

// Parse the args string, removing unnecessary spaces, quotes, etc.
// Handles repeated spaces, single quotes inside double quotes, and many such cases.
fn parse_args(args: &str) -> Vec<&str> {
    let mut output: Vec<&str> = Vec::new();

    let mut it = args.chars().enumerate();
    while let Some((ind1, c)) = it.next() {
        if c == '\'' || c == '\"' {
            let (ind2, _) = it.find(|(_, x)| x == &c).unwrap();

            // Only push non-empty strings
            if ind2 - ind1 > 1 {
                output.push(&args[(ind1 + 1)..ind2]);
            }
        } else if c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '/' || c == '.' {
            // We need to clone as we can't iterate one step back.
            // Hence, create a second iterator and move it as desired,
            // then, move the original iterator one less times than the second iterator
            let (ind2, _) = it
                .clone() // Cheap to clone
                .find(|(_, x)| x == &'\'' || x == &'\"' || x == &' ')
                .unwrap_or((args.len(), ' '));

            output.push(&args[ind1..ind2]);

            // As we can't iterate one step back,
            // we move the original iterator n -1 times
            for _ in 0..(ind2 - ind1 - 1) {
                it.next();
            }
        } else if c == ' ' && !output.is_empty() && output.last().unwrap() != &" " {
            output.push(" ");
        }
    }
    output
}

fn main() -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut input = String::new();
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap(); // Wait for user input

        // Sanitize the input
        let trimmed_input = input.trim(); // Trim also removes the \n when pressing enter to run the command
        if trimmed_input.is_empty() {
            continue;
        }
        // Handle the case of only the command supplied
        let (command, args) = trimmed_input.split_once(" ").unwrap_or((trimmed_input, ""));

        // Clean the args
        let parsed_args = parse_args(args);
        // println!("{:?} {}", parsed_args, parsed_args.len());

        match command {
            "exit" => return ExitCode::from(get_exit_code(parsed_args.first())),
            "echo" => builtin_echo(parsed_args),
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "cd" => builtin_cd(parsed_args.first()),
            "type" => builtin_type(args),
            _ => {
                // Run the command in a subshell if found in PATH
                if get_absolute_command_path(command).is_some() {
                    let _ = Command::new(command)
                        .args(parsed_args.into_iter().filter(|&x| !x.trim().is_empty()))
                        .spawn()
                        .expect("Command failed to run")
                        .wait();
                } else {
                    println!("{}: command not found", trimmed_input)
                }
            }
        }

        input.clear(); // Clear the input string so that it can be used again without re-declaring the variable
    }
}
