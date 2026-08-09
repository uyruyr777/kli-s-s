use crate::runtime::Runtime;
use crate::value::Value;

pub fn load(runtime: &mut Runtime) {

    runtime.namespace("cons")
       .function("print", cons_print)
       .function("println", cons_println);

    runtime.namespace("time")
       .function("sleep", time_sleep);

    /// runtime.namespace("cons", fn);

}

fn cons_print(args: Vec<Value>) -> Value {

    for value in args {

        print!("{}", value);

    }

    Value::Null

}

fn cons_println(args: Vec<Value>) -> Value {

    for value in args {

        print!("{}", value);

    }

    println!();

    Value::Null

}

fn time_sleep(args: Vec<Value>) -> Value {

    if args.len() != 1 {

        panic!("time.sleep(ms)");

    }

    let ms = args[0].as_int();

    std::thread::sleep(

        std::time::Duration::from_millis(ms as u64)

    );

    Value::Null

}

fn file_exists(args: Vec<Value>) -> Value {

    if args.len() != 1 {

        panic!("file.exists(path)");

    }

    let path = args[0].as_string();

    Value::Bool(

        std::path::Path::new(&path).exists()

    )

}

fn file_read(args: Vec<Value>) -> Value {

    if args.len() != 1 {

        panic!("file.read(path)");

    }

    let path = args[0].as_string();

    let text = std::fs::read_to_string(path)

        .unwrap_or_default();

    Value::String(text)

}

fn file_write(args: Vec<Value>) -> Value {

    if args.len() != 2 {

        panic!("file.write(path,text)");

    }

    let path = args[0].as_string();

    let text = args[1].as_string();

    std::fs::write(path, text)

        .expect("Ошибка записи файла");

    Value::Null

}

/// Опрашивается интерпретатором: если пользователь ввёл строку в консоль —
/// возвращает её как значение события; если поток ввода закрыт (EOF) — None.
fn cons_poll_input() -> Option<Value> {

    let mut line = String::new();

    match std::io::stdin().read_line(&mut line) {
        Ok(0) => None, // EOF — поток ввода закрыт
        Ok(_) => {
            let line = line.trim_end_matches(['\n', '\r']).to_string();
            Some(Value::String(line))
        }
        Err(_) => None,
    }

}