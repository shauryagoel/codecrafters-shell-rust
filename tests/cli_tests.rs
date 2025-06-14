use std::env;
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};

#[test]
fn test_echo() {
    // Table drive test
    let table_test_inputs: Vec<(&str, &str)> = vec![
        // Single and double quotes
        ("echo 2", "2"),
        ("echo a", "a"),
        ("echo banana orange", "banana orange"),
        ("echo 'abc'", "abc"),
        ("echo \"abc\"", "abc"),
        ("echo 'shell hello'", "shell hello"),
        ("echo 'world     test'", "world     test"),
        ("echo \"abc\" '123'", "abc 123"),
        ("echo \"quz  hello\"  \"bar\"", "quz  hello bar"),
        ("echo \"bar\"  \"shell's\"  \"foo\"", "bar shell's foo"),
        ("echo \"1\"\"2\"", "12"),
        (
            "echo \"test\"  \"hello's\"  world\"\"script",
            "test hello's worldscript",
        ),
        // Backlash outside quotes
        (r#"echo "before\   after""#, r"before\   after"),
        (r"echo world\ \ \ \ \ \ script", "world      script"),
        // Backslash within double quotes
        (r#"echo "hello'script'\\n'world""#, r"hello'script'\n'world"),
        (
            r#"echo "hello\"insidequotes"script\""#,
            r#"hello"insidequotesscript""#,
        ),
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
    // Pass all the inputs from the table drive test to stdin
    // Then submit `exit` at the end to exit from our shell
    for (input, _) in table_test_inputs.iter() {
        stdin
            .write_all((input.to_string() + "\n").as_bytes())
            .unwrap();
    }
    stdin.write_all(b"exit\n").unwrap();

    let output = child.wait_with_output().expect("failed to read stdout");
    // println!("status::: {}", output.status);
    assert!(output.status.success());

    // Compare the desired results from the stdout with the table of inputs
    for (line, (_, want)) in output.stdout.lines().zip(table_test_inputs) {
        let got = &(line.unwrap())[2..]; // Remove the `$ ` (the prompt)
        assert_eq!(got, want);
    }
}
