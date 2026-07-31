use std::collections::HashMap;

use crate::value::Value;

/// Стек областей видимости: одна "область" на программу/блок/вызов функции.
pub struct Scope {
    frames: Vec<HashMap<String, Value>>,
}

impl Scope {
    pub fn new() -> Self {
        Self { frames: vec![HashMap::new()] }
    }

    /// Войти в новый блок ({...}) — своя область видимости
    pub fn push(&mut self) {
        self.frames.push(HashMap::new());
    }

    /// Выйти из блока
    pub fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Объявить новую переменную в текущей (самой внутренней) области
    pub fn declare(&mut self, name: &str, value: Value) {
        self.frames
            .last_mut()
            .expect("нет активной области видимости")
            .insert(name.to_string(), value);
    }

    /// Присвоить значение уже существующей переменной, ищем от внутренней области к внешней.
    /// Если переменной нигде нет — создаём её в текущей области (мягкое поведение).
    pub fn set(&mut self, name: &str, value: Value) {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) {
                frame.insert(name.to_string(), value);
                return;
            }
        }
        self.declare(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for frame in self.frames.iter().rev() {
            if let Some(value) = frame.get(name) {
                return Some(value.clone());
            }
        }
        None
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
