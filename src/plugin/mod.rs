pub mod discord;

use crate::runtime::Runtime;

pub fn load(runtime: &mut Runtime, name: &str) -> Result<(), String> {

    match name {

        "discord" => {
            discord::load(runtime);
            Ok(())
        }

        _ => Err(format!("Plugin '{}' not found", name))

    }

}
