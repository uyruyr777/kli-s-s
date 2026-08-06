use std::collections::HashMap;

use crate::value::Value;

pub struct Scope {
    frames: Vec<HashMap<String, Value>>,
}

impl Scope {
    pub fn new() -> Self {
        Self { frames: vec![HashMap::new()] }
    }

    pub fn push(&mut self) {
        self.frames.push(HashMap::new());
    }

    /// Выйти из блока
    pub fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    pub fn declare(&mut self, name: &str, value: Value) {
        self.frames
            .last_mut()
            .expect("no active scope")
            .insert(name.to_string(), value);
    }

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
