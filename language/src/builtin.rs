use crate::runtime::Runtime;
use crate::value::Value;

/// Регистрирует функции, которые доступны всегда — без `i:` (ядра) и `a:` (плагины).
/// Математика теперь в отдельном ядре (`kernel/math.rs`, подключается через `i:math;`) —
/// здесь остаются только базовые операции над строками/массивами.
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
        _ => panic!("arr.len ожидает массив"),
    }
}
