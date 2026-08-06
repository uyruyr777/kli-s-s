use crate::runtime::Runtime;
use crate::value::Value;

pub fn load(runtime: &mut Runtime) {
    runtime
        .namespace("discord")
        .function("send", discord_send);
}

fn discord_send(_args: Vec<Value>) -> Value {
    Value::Null
}
