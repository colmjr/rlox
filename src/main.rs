use rlox::{error::LoxError, scanner::Scanner};

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("Usage: lox [script]");
            std::process::exit(64)
        }
    }
}

fn run_file(path: &str) {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("could not read {path}: {e}");
        std::process::exit(74); // IO Error
    });

    let source: Vec<char> = source.chars().collect();

    if let Err(err) = run(&source) {
        for e in &err {
            eprintln!("{e}");
        }
        std::process::exit(65);
    }
}

fn run_prompt() {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();

        match std::io::stdin().read_line(&mut line) {
            Ok(0) => break, // C-d
            Ok(_) => {
                let chars: Vec<char> = line.chars().collect();
                if let Err(err) = run(&chars) {
                    for e in &err {
                        eprintln!("{e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("IO Error {e}")
            }
        }
    }
}

fn run<'a>(source: &'a [char]) -> Result<(), Vec<LoxError>> {
    let mut scanner: Scanner = Scanner::new(source);
    let (tokens, err) = scanner.scan_tokens();
    for token in tokens {
        println!("{token:?}");
    }
    if err.is_empty() { Ok(()) } else { Err(err) }
}
