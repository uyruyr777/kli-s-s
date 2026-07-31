use crate::runtime::Runtime;
use crate::value::Value;

/// ЗАГЛУШКА: замени на реальную реализацию, когда она будет готова.
/// Нужна только для того, чтобы `plugin/mod.rs` (который её подключает) компилировался.
pub fn load(runtime: &mut Runtime) {
    runtime
        .namespace("discord")
        .function("send", discord_send);
}

fn discord_send(_args: Vec<Value>) -> Value {
    // TODO: реальная отправка сообщения в Discord
    Value::Null
}