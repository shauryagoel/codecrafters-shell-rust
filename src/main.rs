#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Wait for user input
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut input = String::new();
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap();

        println!("{}: command not found", input.trim());
        input.clear();
    }
}
