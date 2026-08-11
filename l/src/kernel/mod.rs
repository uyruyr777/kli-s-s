pub mod math;
pub mod render;
pub mod system;

use crate::runtime::Runtime;

pub fn load(runtime: &mut Runtime, name: &str) -> Result<(), String> {
    match name {
        "system" => {
            system::load(runtime);
            Ok(())
        }

        "math" => {
            math::load(runtime);
            Ok(())
        }

        "render" => {
            render::load(runtime);
            Ok(())
        }

        _ => Err(format!("Kernel '{}' not found", name)),
    }
}
