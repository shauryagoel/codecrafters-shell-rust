use std::{
    io::{BufRead, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, ExitCode, Stdio},
};

pub struct Shell<'a> {
    pub command: &'a str,
    args: &'a Vec<&'a str>,
    stdout_buffer: String,
    stdout_stream: Box<dyn Write>,
    stderr_buffer: String,
    stderr_stream: Box<dyn Write>,
}

impl<'a> Shell<'a> {
    pub fn new(
        command: &'a str,
        args: &'a Vec<&'a str>,
        stdout_stream: Box<dyn Write>,
        stderr_stream: Box<dyn Write>,
    ) -> Self {
        Self {
            command,
            args,
            stdout_buffer: String::new(),
            stdout_stream,
            stderr_buffer: String::new(),
            stderr_stream,
        }
    }

    // Parse error code from the Optional str (will be None if the exit command was not given any exit code)
    // Set default of 0 status code if received no status code
    // If unable to parse the string of status code to u8 then give 1 status code
    fn parse_exit_code(&self) -> u8 {
        self.args.first().unwrap_or(&"0").parse::<u8>().unwrap_or(1)
    }

    // Return absolute path of the command if found in the PATH environment variable
    fn get_absolute_command_path(command_name: &str) -> Option<String> {
        let path_directories = std::env::var("PATH").unwrap();
        for directory in path_directories.split(":") {
            // Check if directory actually exists in the filesystem
            let directory_entries = match std::fs::read_dir(directory) {
                Ok(x) => x,
                Err(_) => continue,
            };
            for file in directory_entries {
                // Check permission bits to find executables
                let is_executable = file
                    .as_ref()
                    .unwrap()
                    .metadata()
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o111
                    != 0;
                let file_name = file.as_ref().unwrap().file_name().into_string().unwrap();
                if is_executable && file_name == command_name {
                    return Some(file.unwrap().path().to_str().unwrap().to_owned());
                }
            }
        }
        None
    }

    // Print the vector of strings to stdout
    // This function assumes that the `parsed_args` is already in a clean state
    // like removing duplicate spaces, handling quotes, etc.
    fn builtin_echo(&mut self) -> ExitCode {
        for arg in self.args {
            self.stdout_buffer += arg;
        }
        self.stdout_buffer += "\n";
        ExitCode::SUCCESS
    }

    // Print the current working directory
    fn builtin_pwd(&mut self) -> ExitCode {
        let current_dir = std::env::current_dir().unwrap();
        self.stdout_buffer += current_dir.to_str().unwrap();
        self.stdout_buffer += "\n";
        ExitCode::SUCCESS
    }

    // Change the directory
    fn builtin_cd(&mut self) -> ExitCode {
        // TODO: Maybe try `Cow` to avoid using String
        let corrected_path = if self.args.first().unwrap_or(&"~") == &"~" {
            std::env::var("HOME").unwrap()
        } else {
            (*self.args.first().unwrap()).to_owned()
        };

        let path_obj = Path::new(corrected_path.as_str());

        // Changes the current directory
        // if error occurs, like no directory exists, then, print an error
        if std::env::set_current_dir(path_obj).is_err() {
            self.stdout_buffer += &format!("cd: {}: No such file or directory\n", corrected_path);
            return ExitCode::from(1); // Missing directory error code
        }
        ExitCode::SUCCESS
    }

    // The `type` command
    fn builtin_type(&mut self) -> ExitCode {
        // Command implemented in the current program are called builtin commands
        // NOTE: If adding new builtin command, make sure to add it below
        // TODO: use enums for this to reduce human errors
        let builtin_commands = ["exit", "echo", "type", "pwd", "cd"];

        let first_arg = self.args.first().unwrap();

        if builtin_commands.contains(first_arg) {
            self.stdout_buffer += &format!("{} is a shell builtin\n", first_arg);
        } else if let Some(command_path) = Shell::get_absolute_command_path(first_arg) {
            self.stdout_buffer += &format!("{} is {}\n", first_arg, command_path)
        } else {
            self.stdout_buffer += &format!("{}: not found\n", first_arg);
            return ExitCode::from(1);
        }
        ExitCode::SUCCESS
    }

    // Write the output buffer to the output stream
    pub fn write_to_stdout_buffer(&mut self) {
        self.stdout_stream.write_all(self.stdout_buffer.as_bytes());
    }

    // Write the error buffer to the error stream
    pub fn write_to_stderr_buffer(&mut self) {
        self.stderr_stream.write_all(self.stderr_buffer.as_bytes());
    }

    // Execute the command with args and return appropriate status code
    pub fn execute(&mut self) -> ExitCode {
        match self.command {
            "exit" => ExitCode::from(self.parse_exit_code()),
            "echo" => self.builtin_echo(),
            "pwd" => self.builtin_pwd(),
            "cd" => self.builtin_cd(),
            "type" => self.builtin_type(),
            _ => {
                // Run the command in a subshell if found in PATH
                if Shell::get_absolute_command_path(self.command).is_some() {
                    let child = Command::new(self.command)
                        .args(self.args.iter().filter(|&x| !x.trim().is_empty()))
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Command failed to run");

                    let output = child.wait_with_output().expect("failed to read stdout");

                    // Append Vec<u8> to `stdout_buffer` String
                    // For stdout
                    for line in output.stdout.lines() {
                        self.stdout_buffer += line.unwrap().as_str();
                        self.stdout_buffer += "\n";
                    }
                    // For stderr
                    for line in output.stderr.lines() {
                        self.stderr_buffer += line.unwrap().as_str();
                        self.stderr_buffer += "\n";
                    }

                    (output.status.code().unwrap() as u8).into() // Convert ExitStatus to ExitCode
                } else {
                    self.stdout_buffer += &format!("{}: command not found\n", self.command);
                    ExitCode::from(127) // Command not found error code
                }
            }
        }
    }
}
