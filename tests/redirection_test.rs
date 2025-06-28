use std::env;
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};

#[test]
fn test_stdout_redirection() {
    let input_sequence: Vec<&str> = vec![
        // Set up the test
        "mkdir -p /tmp/baz",
        "touch /tmp/baz/apple /tmp/baz/blueberry",
        "echo blueberry > /tmp/baz/blueberry",
        // Start running commands for tests
        "ls /tmp/baz > /tmp/baz2.md",
        "cat /tmp/baz2.md",
        "echo 'Hello James' 1> /tmp/foo2.md",
        "cat /tmp/foo2.md",
        "cat /tmp/baz/blueberry nonexistent 1> /tmp/quz2.md",
        "cat /tmp/quz2.md",
    ];

    let expected_output_sequence: Vec<&str> =
        vec!["apple", "blueberry", "Hello James", "blueberry"];

    let expected_error_sequence: Vec<&str> = vec!["cat: nonexistent: No such file or directory"];

    let binary_path = env::var("CARGO_MANIFEST_DIR").unwrap()
        + "/target/debug/"
        + env::var("CARGO_PKG_NAME").unwrap().as_ref();
    // println!("{binary_path}");

    let mut child = Command::new(binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start child process");

    let stdin = child.stdin.as_mut().expect("failed to open stdin");
    // Pass all the inputs from the inputs
    // Then submit `exit` at the end to exit from our shell
    for input in input_sequence.iter() {
        stdin
            .write_all((input.to_string() + "\n").as_bytes())
            .unwrap();
    }
    stdin.write_all(b"exit\n").unwrap();

    let output = child.wait_with_output().expect("failed to read stdout");
    // println!("status::: {}", output.status);
    assert!(output.status.success());

    // Compare the desired results from the stdout with the table of inputs
    for (line, want) in output.stdout.lines().zip(expected_output_sequence) {
        let got = line.unwrap();
        assert_eq!(got, want);
    }

    for (line, want) in output.stderr.lines().zip(expected_error_sequence) {
        let got = line.unwrap();
        assert_eq!(got, want);
    }
}

#[test]
fn test_stderr_redirection() {
    let input_sequence: Vec<&str> = vec![
        "ls nonexistent 2> /tmp/baz.md",
        "cat /tmp/baz.md",
        "echo 'Maria file cannot be found' 2> /tmp/foo.md",
        "echo 'pear' > /tmp/pear",
        "cat /tmp/pear nonexistent 2> /tmp/quz.md",
        "cat /tmp/quz.md",
    ];

    let expected_output_sequence: Vec<&str> = vec![
        "ls: nonexistent: No such file or directory",
        "Maria file cannot be found",
        "pear",
        "cat: nonexistent: No such file or directory",
    ];

    let binary_path = env::var("CARGO_MANIFEST_DIR").unwrap()
        + "/target/debug/"
        + env::var("CARGO_PKG_NAME").unwrap().as_ref();
    // println!("{binary_path}");

    let mut child = Command::new(binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start child process");

    let stdin = child.stdin.as_mut().expect("failed to open stdin");
    // Pass all the inputs from the inputs
    // Then submit `exit` at the end to exit from our shell
    for input in input_sequence.iter() {
        stdin
            .write_all((input.to_string() + "\n").as_bytes())
            .unwrap();
    }
    stdin.write_all(b"exit\n").unwrap();

    let output = child.wait_with_output().expect("failed to read stdout");
    // println!("status::: {}", output.status);
    assert!(output.status.success());

    // Compare the desired results from the stdout with the table of inputs
    for (line, want) in output.stdout.lines().zip(expected_output_sequence) {
        let got = line.unwrap();
        assert_eq!(got, want);
    }
}

#[test]
fn test_stdout_append() {
    let input_sequence: Vec<&str> = vec![
        // Set up the test
        "mkdir -p /tmp/stdout_append /tmp/stdout_append_test",
        "touch /tmp/stdout_append/apple /tmp/stdout_append/banana /tmp/stdout_append/blueberry",
        // Start running commands for tests
        "ls /tmp/stdout_append >> /tmp/stdout_append_test/bar.md",
        "cat /tmp/stdout_append_test/bar.md",
        "echo 'Hello Emily' 1>> /tmp/stdout_append_test/baz.md",
        "echo 'Hello Maria' 1>> /tmp/stdout_append_test/baz.md",
        "cat /tmp/stdout_append_test/baz.md",
        r#"echo "List of files: " > /tmp/stdout_append_test/qux.md"#,
        "ls /tmp/stdout_append >> /tmp/stdout_append_test/qux.md",
        "cat /tmp/stdout_append_test/qux.md",
        // Clean up the test
        "rm -rf /tmp/stdout_append /tmp/stdout_append_test",
    ];

    let expected_output_sequence: Vec<&str> = vec![
        "apple",
        "banana",
        "blueberry",
        "Hello Emily",
        "Hello Maria",
        "List of files: ",
        "apple",
        "banana",
        "blueberry",
    ];

    let expected_error_sequence: Vec<&str> = vec![];

    let binary_path = env::var("CARGO_MANIFEST_DIR").unwrap()
        + "/target/debug/"
        + env::var("CARGO_PKG_NAME").unwrap().as_ref();
    // println!("{binary_path}");

    let mut child = Command::new(binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start child process");

    let stdin = child.stdin.as_mut().expect("failed to open stdin");
    // Pass all the inputs from the inputs
    // Then submit `exit` at the end to exit from our shell
    for input in input_sequence.iter() {
        stdin
            .write_all((input.to_string() + "\n").as_bytes())
            .unwrap();
    }
    stdin.write_all(b"exit\n").unwrap();

    let output = child.wait_with_output().expect("failed to read stdout");
    // println!("status::: {}", output.status);
    assert!(output.status.success());

    // Compare the desired results from the stdout with the table of inputs
    for (line, want) in output.stdout.lines().zip(expected_output_sequence) {
        let got = line.unwrap();
        assert_eq!(got, want);
    }

    for (line, want) in output.stderr.lines().zip(expected_error_sequence) {
        let got = line.unwrap();
        assert_eq!(got, want);
    }
}

#[test]
fn test_stderr_append() {
    let input_sequence: Vec<&str> = vec![
        // Set up the test
        "mkdir -p /tmp/stderr_append",
        // Start running commands for tests
        "ls nonexistent >> /tmp/stderr_append/baz.md",
        "ls nonexistent 2>> /tmp/stderr_append/qux.md",
        "cat /tmp/stderr_append/qux.md",
        r#"echo "James says Error" 2>> /tmp/stderr_append/quz.md"#,
        "cat nonexistent 2>> /tmp/stderr_append/quz.md",
        "ls nonexistent 2>> /tmp/stderr_append/quz.md",
        "cat /tmp/stderr_append/quz.md",
        // Clean up the test
        "rm -rf /tmp/stderr_append",
    ];

    let expected_output_sequence: Vec<&str> = vec![
        "ls: nonexistent: No such file or directory",
        "James says Error",
        "cat: nonexistent: No such file or directory",
        "ls: nonexistent: No such file or directory",
    ];

    let expected_error_sequence: Vec<&str> = vec!["ls: nonexistent: No such file or directory"];

    let binary_path = env::var("CARGO_MANIFEST_DIR").unwrap()
        + "/target/debug/"
        + env::var("CARGO_PKG_NAME").unwrap().as_ref();
    // println!("{binary_path}");

    let mut child = Command::new(binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start child process");

    let stdin = child.stdin.as_mut().expect("failed to open stdin");
    // Pass all the inputs from the inputs
    // Then submit `exit` at the end to exit from our shell
    for input in input_sequence.iter() {
        stdin
            .write_all((input.to_string() + "\n").as_bytes())
            .unwrap();
    }
    stdin.write_all(b"exit\n").unwrap();

    let output = child.wait_with_output().expect("failed to read stdout");
    // println!("status::: {}", output.status);
    assert!(output.status.success());

    // Compare the desired results from the stdout with the table of inputs
    for (line, want) in output.stdout.lines().zip(expected_output_sequence) {
        let got = line.unwrap();
        assert_eq!(got, want);
    }

    for (line, want) in output.stderr.lines().zip(expected_error_sequence) {
        let got = line.unwrap();
        assert_eq!(got, want);
    }
}
