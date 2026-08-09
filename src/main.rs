mod ast;
mod builtin;
mod interpreter;
mod kernel;
mod lexer;
mod parser;
mod plugin;
mod runtime;
mod scope;
mod value;

use std::fs;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use runtime::Runtime;

fn main() {
    let path = parse_args();

    println!("==========================");
    println!("KLI-S-S (KSS) v0.1");
    println!("==========================");

    let source = match fs::read_to_string(&path) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Could not open '{}': {}", path, err);
            std::process::exit(1);
        }
    };

    println!("File '{}' loaded successfully.", path);

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let program = parser.parse();

    let mut runtime = Runtime::new();
    builtin::load(&mut runtime);

    for import in &program.imports {
        for name in &import.names {
            if let Err(err) = kernel::load(&mut runtime, name) {
                eprintln!("Error loading kernel: {}", err);
                std::process::exit(1);
            }
        }
    }

    for import in &program.plugins {
        for name in &import.names {
            if let Err(err) = plugin::load(&mut runtime, name) {
                eprintln!("Error loading plugin: {}", err);
                std::process::exit(1);
            }
        }
    }

    let mut interpreter = Interpreter::new(runtime);
    interpreter.run(&program);
}

fn parse_args() -> String {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-x" => {
                return match args.get(i + 1) {
                    Some(path) => path.clone(),
                    None => {
                        eprintln!("Error: -x must be followed by a path to a .kss file");
                        print_usage();
                        std::process::exit(1);
                    }
                };
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    print_usage();
    std::process::exit(1);
}

fn print_usage() {
    eprintln!("Usage: kli-s-s -x <file.kss>");
    eprintln!("Example:      kli-s-s -x game.kss");
}
