pub mod discord;

use crate::runtime::Runtime;

/// Загрузить внешний плагин
pub fn load(runtime: &mut Runtime, name: &str) -> Result<(), String> {

    match name {

        "discord" => {
            discord::load(runtime);
            Ok(())
        }

        _ => Err(format!("Плагин '{}' не найден", name))

    }

}