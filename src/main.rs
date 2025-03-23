#[allow(unused_imports)]
use std::io::{self, Write};

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;

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

fn echo(args: &str) {
    let parsed_string: String = if args.starts_with('\'') && args.ends_with('\'') {
        args.split('\'').filter(|&x| !x.is_empty()).collect()
    } else {
        args.split_ascii_whitespace()
            .map(|x| String::from(x) + " ")
            .collect()
    };
    println!("{}", parsed_string);
}

// Function to change the directory (implemention of cd)
fn change_directory(path: &str) {
    // TODO: Maybe try `Cow` to avoid using String
    let cleaned_path = if path == "~" {
        env::var("HOME").unwrap()
    } else {
        String::from(path)
    };

    let path_obj = Path::new(cleaned_path.as_str());

    // Changes the current directory, if error occurs like no directory exists, then, print an error
    if env::set_current_dir(path_obj).is_err() {
        println!("cd: {}: No such file or directory", path);
    }
}

fn main() -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let builtin_commands = ["exit", "echo", "type", "pwd", "cd"];

    let mut input = String::new();
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap(); // Wait for user input

        // Sanitize the input
        let trimmed_input = input.trim(); // Trim also removes the \n when pressing enter to run the command
                                          // let mut trimmed_input_it = trimmed_input.split_ascii_whitespace();
                                          // let command = trimmed_input_it.next().unwrap();
                                          // let mut args: String = trimmed_input_it.map(|x| String::from(x) + " ").collect();
                                          // args.pop(); // Remove the extra space at the end

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
        let parsed_args: Vec<_> = args.split('\'').filter(|&x| !x.is_empty()).collect();

        match command {
            "exit" => return ExitCode::from(args.as_bytes()[0] - 48), // Return exit code from the program and exit it TODO: use parse
            "echo" => echo(args),
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "cd" => change_directory(args),
            "type" => {
                if builtin_commands.contains(&args) {
                    println!("{} is a shell builtin", args);
                } else if let Some(command_path) = get_path(args) {
                    println!("{} is {}", args, command_path)
                } else {
                    println!("{}: not found", args);
                }
            }
            _ => {
                // Run arbitrary command if found in PATH
                if get_path(command).is_some() {
                    let _ = Command::new(command)
                        .args(parsed_args.iter().filter(|&&x| !x.trim().is_empty()))
                        .spawn()
                        .expect("Command failed to run")
                        .wait();
                } else {
                    // This is done instead of using trimmed_input because it may not be sanitized
                    let mut message = String::from(command);
                    if !args.is_empty() {
                        message += " {}";
                    }
                    println!("{}: command not found", message)
                }
            }
        }

        input.clear(); // Clear the input string so that it can be used again without re-declaring the variable
    }
}
