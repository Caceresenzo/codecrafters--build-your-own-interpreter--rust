use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

use interpreter_starter_rust::{Interpreter, Literal, Parser, Scanner};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            let mut scanner = Scanner::new(file_contents);
            let tokens = scanner.scan_tokens();

            for token in tokens {
                println!("{}", token);
            }

            if scanner.had_error {
                exit(65);
            }
        }
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            let mut scanner = Scanner::new(file_contents);
            let tokens = scanner.scan_tokens();

            if scanner.had_error {
                exit(65);
            }

            let mut parser = Parser::new(tokens);
            match parser.expression() {
                Ok(root) => println!("{}", root),
                Err(error) => {
                    eprintln!("{error}");
                    exit(65);
                }
            }
        }
        "evaluate" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            let mut scanner = Scanner::new(file_contents);
            let tokens = scanner.scan_tokens();

            if scanner.had_error {
                exit(65);
            }

            let mut parser = Parser::new(tokens);
            let root = match parser.expression() {
                Ok(root) => root,
                Err(error) => {
                    eprintln!("{error}");
                    exit(65);
                }
            };

            let interpreter = Interpreter::new();
            match interpreter.evaluate(root) {
                Ok(value) => match value {
                    Literal::Number(x) => println!("{}", x),
                    _ => println!("{}", value),
                },
                Err(error) => {
                    eprintln!("{error}");

                    if let Some(token) = error.token {
                        eprintln!("[line {}]", token.line);
                    }

                    exit(70);
                }
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
