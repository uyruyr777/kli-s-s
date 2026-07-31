pub mod math;
pub mod system;
///pub mod scretch;

use crate::runtime::Runtime;

/// Загрузить встроенное ядро по имени (вызывается для каждого имени из `i:...;`)
pub fn load(runtime: &mut Runtime, name: &str) -> Result<(), String> {
    match name {
        "system" => {
            system::load(runtime);
            Ok(())
        }

        ///"scretch" => {
        ///    scretch::load(runtime);
        ///    Ok(())
        /// }

        "math" => {
            math::load(runtime);
            Ok(())
        }

        _ => Err(format!("Ядро '{}' не найдено", name)),
    }
}
