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
fn get_path(command_name: &str) -> Option<String> {
    let path_directories = env::var("PATH").unwrap();
    for directory in path_directories.split(":") {
        for file in fs::read_dir(directory).unwrap() {
            let file_name = file.as_ref().unwrap().file_name().into_string().unwrap();
            if file_name == command_name {
                return Some(file.as_ref().unwrap().path().to_str().unwrap().to_owned());
            }
        }
    }
    None
}

fn builtin_echo(args: &str) {
    // Handle case of args which contains double single quotes and also no quotes
    let parsed_string: String = if args.starts_with('\'') && args.ends_with('\'') {
        args.split('\'').filter(|&x| !x.is_empty()).collect()
    } else {
        args.split_ascii_whitespace()
            .map(|x| String::from(x) + " ")
            .collect()
    };
    println!("{}", parsed_string);
}

fn builtin_cd(path: Option<&&str>) {
    // TODO: Maybe try `Cow` to avoid using String
    let corrected_path = if path.unwrap_or(&"~") == &"~" {
        env::var("HOME").unwrap()
    } else {
        path.unwrap().to_owned().to_owned()
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
    let builtin_commands = ["exit", "echo", "type", "pwd", "cd"];

    if builtin_commands.contains(&command) {
        println!("{} is a shell builtin", command);
    } else if let Some(command_path) = get_path(command) {
        println!("{} is {}", command, command_path)
    } else {
        println!("{}: not found", command);
    }
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
        let command_args = trimmed_input.split_once(" ");

        if command_args.is_none() && trimmed_input.is_empty() {
            continue;
        }

        // Handle the case of only the command supplied
        let (command, args) = if command_args.is_none() && !trimmed_input.is_empty() {
            (trimmed_input, "")
        } else {
            command_args.unwrap()
        };

        // Handle double single quotes in args
        let parsed_args: Vec<&str> = args.split('\'').filter(|&x| !x.is_empty()).collect();
        // println!("{:?}", parsed_args);

        match command {
            "exit" => return ExitCode::from(get_exit_code(parsed_args.first())),
            "echo" => builtin_echo(args),
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "cd" => builtin_cd(parsed_args.first()),
            "type" => builtin_type(args),
            _ => {
                // Run the command in a subshell if found in PATH
                if get_path(command).is_some() {
                    let _ = Command::new(command)
                        .args(parsed_args.iter().filter(|&&x| !x.trim().is_empty()))
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
