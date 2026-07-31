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

    // Загружаем файл программы
    let source = match fs::read_to_string(&path) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Не удалось открыть '{}': {}", path, err);
            std::process::exit(1);
        }
    };

    println!("Файл '{}' успешно загружен.", path);

    // Лексер -> токены
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Токены -> AST
    let mut parser = Parser::new(tokens);
    let program = parser.parse();

    // Runtime: сначала встроенные функции (доступны всегда, без i:/a:)
    let mut runtime = Runtime::new();
    builtin::load(&mut runtime);

    // Ядра, подключённые через `i:...;`
    for import in &program.imports {
        for name in &import.names {
            if let Err(err) = kernel::load(&mut runtime, name) {
                eprintln!("Ошибка загрузки ядра: {}", err);
                std::process::exit(1);
            }
        }
    }

    // Плагины, подключённые через `a:...;`
    for import in &program.plugins {
        for name in &import.names {
            if let Err(err) = plugin::load(&mut runtime, name) {
                eprintln!("Ошибка загрузки плагина: {}", err);
                std::process::exit(1);
            }
        }
    }

    // Исполняем программу
    let mut interpreter = Interpreter::new(runtime);
    interpreter.run(&program);
}

/// Разбор аргументов командной строки.
/// Использование: `kli-s-s -x путь/к/файлу.kss`
fn parse_args() -> String {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-x" => {
                return match args.get(i + 1) {
                    Some(path) => path.clone(),
                    None => {
                        eprintln!("Ошибка: после -x нужно указать путь к .kss файлу");
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
                eprintln!("Неизвестный аргумент: {}", other);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    print_usage();
    std::process::exit(1);
}

fn print_usage() {
    eprintln!("Использование: kli-s-s -x <файл.kss>");
    eprintln!("Пример:        kli-s-s -x game.kss");
}
