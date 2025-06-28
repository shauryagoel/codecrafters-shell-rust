use std::fs::OpenOptions;
use std::{
    fs::File,
    io::{self, Write},
    process::ExitCode,
};

mod shell;
use shell::Shell;

// Handle redirection of output of a command to stdout or to a file
fn get_stdout_stream_path(parsed_args: &mut Vec<&str>) -> Box<dyn Write> {
    // dbg!(&parsed_args);
    let result = parsed_args
        .iter()
        .find(|&&x| x == "1>" || x == ">" || x == ">>" || x == "1>>");
    if result.is_some() && parsed_args.len() >= 3 {
        let file_name = parsed_args.last().unwrap();
        // Choose whether to truncate a file or append to a file
        let file = match result {
            Some(&">") | Some(&"1>") => File::create(file_name).unwrap(), // This is also a wrapper over `OpenOptions`
            Some(&">>") | Some(&"1>>") => OpenOptions::new()
                .append(true)
                .create(true)
                .open(file_name)
                .unwrap(),
            Some(_) | None => panic!("Not possible"),
        };
        parsed_args.pop(); // Remove the file path
        parsed_args.pop(); // Remove the " "
        parsed_args.pop(); // Remove the redirection symbol
        parsed_args.pop(); // Needed for running tests; will be `None` outside tests
        Box::new(file)
    } else {
        Box::new(io::stdout())
    }
}

// Handle redirection of error of a command to stderr or to a file
fn get_stderr_stream_path(parsed_args: &mut Vec<&str>) -> Box<dyn Write> {
    let result = parsed_args.iter().find(|&&x| x == "2>" || x == "2>>");
    if result.is_some() && parsed_args.len() >= 3 {
        let file_name = parsed_args.last().unwrap();
        // Choose whether to truncate a file or append to a file
        let file = match result {
            Some(&"2>") => File::create(file_name).unwrap(), // This is also a wrapper over `OpenOptions`
            Some(&"2>>") => OpenOptions::new()
                .append(true)
                .create(true)
                .open(file_name)
                .unwrap(),
            Some(_) | None => panic!("Not possible"),
        };
        parsed_args.pop(); // Remove the file path
        parsed_args.pop(); // Remove the " "
        parsed_args.pop(); // Remove the redirection symbol
        parsed_args.pop(); // Needed for running tests; will be `None` outside tests
        Box::new(file)
    } else {
        Box::new(io::stderr())
    }
}

// Parse the input string, removing unnecessary spaces, quotes, etc.
// Handles repeated spaces, single quotes inside double quotes, and many such cases.
fn parse_input(args: &str) -> Vec<&str> {
    let mut output: Vec<&str> = Vec::new();

    // Make this peekable to allow peeking the values
    // in the case where the current character is part of a word
    let mut it = args.chars().enumerate().peekable();
    while let Some((ind1, c)) = it.next() {
        if c == '\'' {
            let (ind2, _) = it.find(|(_, x)| x == &'\'').unwrap();
            output.push(&args[(ind1 + 1)..ind2]);
        } else if c == '"' {
            let mut prev_ind1 = ind1 + 1; // After the `"`

            while let Some((_ind2, _c2)) = it.next() {
                if _c2 == '"' {
                    output.push(&args[prev_ind1.._ind2]);
                    break;
                } else if _c2 == '\\' {
                    // Backslash before the below 4 characters preserves the special meaning of these
                    let (_ind3, _c3) = it.next().unwrap();
                    if _c3 == '\n' || _c3 == '$' || _c3 == '"' || _c3 == '\\' {
                        output.push(&args[prev_ind1.._ind2]);
                        output.push(&args[_ind3..(_ind3 + 1)]);
                        prev_ind1 = _ind3 + 1;
                    }
                }
            }
        } else if c.is_ascii_alphabetic()
            || c.is_ascii_digit()
            || c == '/'
            || c == '.'
            || c == '>' // For redirecting to a file
            || c == '-'
        // For `ls -1`
        {
            let mut ind2 = args.len(); // Default value is end of string

            // Check the next value before moving the iterator
            while let Some(&(i, c)) = it.peek() {
                if c == '\'' || c == '"' || c == ' ' || c == '\\' {
                    ind2 = i;
                    break;
                }
                it.next();
            }
            output.push(&args[ind1..ind2]);
        } else if c == ' ' && !output.is_empty() && output.last().unwrap() != &" " {
            output.push(" ");
        } else if c == '\\' {
            // Handle cases like "a\ \ b"
            let _ind = it.next().unwrap().0;
            output.push(&args[_ind..(_ind + 1)]); // Push the &str at _ind
        }
    }
    output
}

fn main() -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();

    let is_testing = false; // NOTE: set to `true` during testing
    loop {
        // Printing shell prompt is disabled during tests
        // to avoid removing `$ ` from stdout
        if !is_testing {
            print!("$ ");
        }
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap(); // Wait for user input

        // Sanitize the input
        let trimmed_input = input.trim(); // Trim also removes the \n when pressing enter to run the command
        if trimmed_input.is_empty() {
            continue;
        }

        // Clean the input
        let parsed_input = parse_input(trimmed_input);
        let parsed_command = parsed_input[0];
        let mut parsed_args: Vec<&str> = if parsed_input.len() > 2 {
            parsed_input[2..].into() // The second element in the parsed_input is " ", hence skip it
        } else {
            Vec::new()
        };

        let stdout_path = get_stdout_stream_path(&mut parsed_args);
        let stderr_path = get_stderr_stream_path(&mut parsed_args);
        let mut shell = Shell::new(parsed_command, &parsed_args, stdout_path, stderr_path); // TODO: move creation of shell outside for loop

        // println!("{:?} {}", parsed_args, parsed_args.len());

        let status_code = shell.execute();

        // Quit the shell if user supplies the `exit` command
        // TODO: create a getter for this
        if shell.command == "exit" {
            return status_code;
        }

        shell.write_to_stdout_buffer(); // Write the output of the command to the output buffer
        shell.write_to_stderr_buffer(); // Write the output of the command to the output buffer
        input.clear(); // Clear the input string so that it can be used again without re-declaring the variable
    }
}
