use crate::runtime::Runtime;
use crate::value::{Ncti, Value};

pub fn load(runtime: &mut Runtime) {
    runtime
        .namespace("math")
        .function("abs", math_abs)
        .function("pow", math_pow)
        .function("sqrt", math_sqrt)
        .function("floor", math_floor)
        .function("ceil", math_ceil)
        .function("round", math_round)
        .function("min", math_min)
        .function("max", math_max)
        .function("sin", math_sin)
        .function("cos", math_cos)
        .function("tan", math_tan)
        .function("log", math_log)
        .function("log10", math_log10)
        .function("exp", math_exp)
        .function("pi", math_pi)
        .function("e", math_e)
        .function("gcd", math_gcd)
        .function("lcm", math_lcm)
        .function("factorial", math_factorial)
        .function("nctiPow", math_ncti_pow);
}

fn math_abs(args: Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Float(_)) => Value::Float(args[0].as_float().abs()),
        Some(v) => Value::Int(v.as_int().abs()),
        None => panic!("math.abs(n)"),
    }
}

fn math_pow(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        panic!("math.pow(base, exponent)");
    }
    Value::Float(args[0].as_float().powf(args[1].as_float()))
}

fn math_sqrt(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("math.sqrt(n)");
    }
    Value::Float(args[0].as_float().sqrt())
}

fn math_floor(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("math.floor(n)");
    }
    Value::Int(args[0].as_float().floor() as i64)
}

fn math_ceil(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("math.ceil(n)");
    }
    Value::Int(args[0].as_float().ceil() as i64)
}

fn math_round(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("math.round(n)");
    }
    Value::Int(args[0].as_float().round() as i64)
}

fn math_min(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        panic!("math.min(a, b)");
    }
    if args[0].as_float() <= args[1].as_float() {
        args[0].clone()
    } else {
        args[1].clone()
    }
}

fn math_max(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        panic!("math.max(a, b)");
    }
    if args[0].as_float() >= args[1].as_float() {
        args[0].clone()
    } else {
        args[1].clone()
    }
}

fn math_sin(args: Vec<Value>) -> Value {
    Value::Float(args[0].as_float().sin())
}

fn math_cos(args: Vec<Value>) -> Value {
    Value::Float(args[0].as_float().cos())
}

fn math_tan(args: Vec<Value>) -> Value {
    Value::Float(args[0].as_float().tan())
}

fn math_log(args: Vec<Value>) -> Value {
    Value::Float(args[0].as_float().ln())
}

fn math_log10(args: Vec<Value>) -> Value {
    Value::Float(args[0].as_float().log10())
}

fn math_exp(args: Vec<Value>) -> Value {
    Value::Float(args[0].as_float().exp())
}

fn math_pi(_args: Vec<Value>) -> Value {
    Value::Float(std::f64::consts::PI)
}

fn math_e(_args: Vec<Value>) -> Value {
    Value::Float(std::f64::consts::E)
}

fn math_gcd(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        panic!("math.gcd(a, b)");
    }
    let mut a = args[0].as_int().abs();
    let mut b = args[1].as_int().abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    Value::Int(a)
}

fn math_lcm(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        panic!("math.lcm(a, b)");
    }
    let a = args[0].as_int();
    let b = args[1].as_int();
    let gcd = math_gcd(vec![Value::Int(a), Value::Int(b)]).as_int();
    if gcd == 0 {
        Value::Int(0)
    } else {
        Value::Int((a / gcd * b).abs())
    }
}

fn math_factorial(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("math.factorial(n)");
    }
    let n = args[0].as_int();
    if n < 0 {
        panic!("math.factorial: the number must not be negative");
    }

    let mut result = Ncti::from_i64(1);
    for i in 1..=n {
        result = result.mul(&Ncti::from_i64(i));
    }

    Value::Ncti(result)
}

fn math_ncti_pow(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        panic!("math.nctiPow(base, exponent)");
    }
    let base = args[0].as_ncti();
    let exponent = args[1].as_int();
    if exponent < 0 {
        panic!("math.nctiPow: negative exponents are not supported");
    }

    let mut result = Ncti::from_i64(1);
    for _ in 0..exponent {
        result = result.mul(&base);
    }

    Value::Ncti(result)
}
