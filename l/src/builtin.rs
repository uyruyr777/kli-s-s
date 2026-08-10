use crate::runtime::Runtime;
use crate::value::Value;

pub fn load(runtime: &mut Runtime) {
    runtime.namespace("str").function("len", str_len);
    runtime.namespace("arr").function("len", arr_len);
}

fn str_len(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("str.len(s)");
    }
    Value::Int(args[0].as_string().len() as i64)
}

fn arr_len(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("arr.len(a)");
    }
    match &args[0] {
        Value::Array(items) => Value::Int(items.len() as i64),
        _ => panic!("arr.len expects an array"),
    }
}
