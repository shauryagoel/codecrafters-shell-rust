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
                return Some(
                    file.as_ref()
                        .unwrap()
                        .path()
                        .into_os_string()
                        .into_string()
                        .unwrap(),
                );
            }
        }
    }
    None
}

// Function to change the directory (implemention of cd)
fn change_directory(path: &str) {
    let path_obj = Path::new(path);

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
        let mut trimmed_input_it = trimmed_input.split_ascii_whitespace();
        let command = trimmed_input_it.next().unwrap();
        let mut args: String = trimmed_input_it.map(|x| String::from(x) + " ").collect();
        args.pop(); // Remove the extra space at the end

        match command {
            "exit" => return ExitCode::from(args.as_bytes()[0] - 48), // Return exit code from the program and exit it
            "echo" => println!("{}", args),
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "cd" => change_directory(args.as_str()),
            "type" => {
                if builtin_commands.contains(&args.as_str()) {
                    println!("{} is a shell builtin", args);
                } else if let Some(command_path) = get_path(args.as_str()) {
                    println!("{} is {}", args, command_path)
                } else {
                    println!("{}: not found", args);
                }
            }
            _ => {
                // Uncomment below 2 lines for using absolute path of the command
                // if let Some(command_path) = get_path(x.0) {
                // let _ = Command::new(command_path)
                // Run arbitrary command if found in PATH
                if get_path(command).is_some() {
                    let _ = Command::new(command)
                        .args(args.split(" "))
                        .spawn()
                        .expect("Command failed to run")
                        .wait();
                } else {
                    // This is done instead of using trimmed_input is because it may not be sanitized
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
