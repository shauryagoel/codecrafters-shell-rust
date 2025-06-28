use std::{
    fs::File,
    io::{self, Write},
    process::ExitCode,
};

mod shell;
use shell::Shell;

// Handle redirection of output of a command to stdout or to a file
fn get_stdout_stream_path(parsed_args: &mut Vec<&str>) -> Box<dyn Write> {
    let result = parsed_args.iter().find(|&&x| x == "1>" || x == ">");
    if result.is_some() && parsed_args.len() >= 3 {
        let new_file = File::create(parsed_args.pop().unwrap()).unwrap();
        parsed_args.pop(); // Remove the " "
        parsed_args.pop(); // Remove the redirection symbol
        parsed_args.pop(); // Remove the " "
        Box::new(new_file)
    } else {
        Box::new(io::stdout())
    }
}

// Handle redirection of error of a command to stderr or to a file
fn get_stderr_stream_path(parsed_args: &mut Vec<&str>) -> Box<dyn Write> {
    let result = parsed_args.iter().find(|&&x| x == "2>");
    if result.is_some() && parsed_args.len() >= 3 {
        let new_file = File::create(parsed_args.pop().unwrap()).unwrap();
        parsed_args.pop(); // Remove the " "
        parsed_args.pop(); // Remove the redirection symbol
        parsed_args.pop(); // Remove the " "
        Box::new(new_file)
    } else {
        Box::new(io::stderr())
    }
}

// Parse the input string, removing unnecessary spaces, quotes, etc.
// Handles repeated spaces, single quotes inside double quotes, and many such cases.
fn parse_input(args: &str) -> Vec<&str> {
    let mut output: Vec<&str> = Vec::new();

    let mut it = args.chars().enumerate();
    while let Some((ind1, c)) = it.next() {
        if c == '\'' {
            let (ind2, _) = it.find(|(_, x)| x == &'\'').unwrap();
            output.push(&args[(ind1 + 1)..ind2]);
        } else if c == '"' {
            let mut prev_ind1 = ind1 + 1; // After the quote
            let it2 = it.by_ref();

            while let Some((_ind2, _c2)) = it2.next() {
                if _c2 == '"' {
                    output.push(&args[prev_ind1.._ind2]);
                    break;
                } else if _c2 == '\\' {
                    let (_ind3, _c3) = it2.next().unwrap();
                    // Backslash before the following 4 characters preserves the special meaning of these
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
            // We need to clone as we can't iterate one step back.
            // Hence, create a second iterator and move it as desired,
            // then, move the original iterator one less times than the second iterator
            let (ind2, _) = it
                .clone() // Cheap to clone
                .find(|(_, x)| x == &'\'' || x == &'\"' || x == &' ' || x == &'\\')
                .unwrap_or((args.len(), ' '));

            output.push(&args[ind1..ind2]);

            // As we can't iterate one step back,
            // we move the original iterator n -1 times
            for _ in 0..(ind2 - ind1 - 1) {
                it.next();
            }
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
