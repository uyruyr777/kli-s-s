use crate::runtime::Runtime;
use crate::value::Value;

pub fn load(runtime: &mut Runtime) {

    runtime.namespace("cons")
       .function("print", cons_print)
       .function("println", cons_println);

    runtime.namespace("file")
       .function("read", file_read)
       .function("write", file_write)
       .function("exists", file_exists);

    runtime.namespace("time")
       .function("sleep", time_sleep);

    runtime.namespace("rand")
       .function("rdom", rand_rdom);

    runtime.event_source("cons", "imput", cons_poll_input);

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

        .expect("Failed to write file");

    Value::Null

}

fn next_random_u64() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};

    static CALLS: AtomicU64 = AtomicU64::new(0);
    let calls = CALLS.fetch_add(1, Ordering::Relaxed);

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut x = time ^ calls.wrapping_mul(0x9E3779B97F4A7C15) ^ 0x2545_F491_4F6C_DD1D;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

fn rand_rdom(args: Vec<Value>) -> Value {

    if args.len() != 2 {
        panic!("rand.rdom(min, max)");
    }

    let min = args[0].as_int();
    let max = args[1].as_int();

    if max < min {
        panic!("rand.rdom: max must not be less than min");
    }

    let range = (max - min + 1) as u64;
    let n = next_random_u64() % range;

    Value::Int(min + n as i64)

}

fn cons_poll_input() -> Option<Value> {

    let mut line = String::new();

    match std::io::stdin().read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => {
            let line = line.trim_end_matches(['\n', '\r']).to_string();
            Some(Value::String(line))
        }
        Err(_) => None,
    }

}
