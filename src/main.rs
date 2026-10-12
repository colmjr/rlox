use rlox::{
    ast::Stmt, environment::Environment, error::LoxError, interpreter::Interpreter, parser::Parser,
    scanner::Scanner,
};

use std::{cell::RefCell, io::Write, process::ExitCode, rc::Rc};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            run_prompt();
            ExitCode::SUCCESS
        }
        2 => run_file(&args[1]),
        _ => {
            eprintln!("Usage: lox [script]");
            ExitCode::from(64)
        }
    }
}

fn run_file(path: &str) -> ExitCode {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("could not read {path}: {e}");
        std::process::exit(74); // IO Error
    });

    let source: Vec<char> = source.chars().collect();

    if let Err(errors) = run(&source) {
        for e in errors {
            eprintln!("{e}");
        }
        return ExitCode::from(65);
    }
    ExitCode::SUCCESS
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

fn run(source: &[char]) -> Result<(), Vec<LoxError>> {
    let mut scanner: Scanner = Scanner::new(source);
    let (tokens, errors) = scanner.scan_tokens();
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut parser = Parser::new(tokens, 0);
    let mut interpreter = Interpreter {
        environment: Rc::new(RefCell::new(Environment::new())),
    };
    let expression: Vec<Stmt> = parser.parse()?;
    interpreter.interpret(expression).map_err(|e| vec![e])?;
    Ok(())
}
