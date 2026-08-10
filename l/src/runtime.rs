use std::collections::HashMap;

use crate::value::Value;

pub type NativeFunction = fn(Vec<Value>) -> Value;

pub type EventSourceFn = fn() -> Option<Value>;

pub type TickFn = fn() -> bool;

pub struct Namespace {
    pub name: String,
    pub functions: HashMap<String, NativeFunction>,
}

impl Namespace {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &str, function: NativeFunction) {
        self.functions.insert(name.to_string(), function);
    }

    pub fn function(&mut self, name: &str, function: NativeFunction) -> &mut Self {
        self.functions.insert(name.to_string(), function);
        self
    }
}

pub struct Runtime {
    pub namespaces: HashMap<String, Namespace>,
    pub event_sources: Vec<(String, String, EventSourceFn)>,
    pub tick_functions: Vec<TickFn>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
            event_sources: Vec::new(),
            tick_functions: Vec::new(),
        }
    }

    pub fn event_source(&mut self, namespace: &str, event: &str, poll: EventSourceFn) {
        self.event_sources.push((namespace.to_string(), event.to_string(), poll));
    }

    pub fn register_tick(&mut self, tick: TickFn) {
        self.tick_functions.push(tick);
    }

    pub fn register_namespace(&mut self, namespace: Namespace) {
        self.namespaces.insert(namespace.name.clone(), namespace);
    }

    pub fn namespace(&mut self, name: &str) -> &mut Namespace {
        self.namespaces
            .entry(name.to_string())
            .or_insert_with(|| Namespace::new(name))
    }

    pub fn call_native(&self, namespace: &str, function: &str, args: Vec<Value>) -> Value {
        let ns = self
            .namespaces
            .get(namespace)
            .expect("Namespace not found");

        let func = ns
            .functions
            .get(function)
            .expect("Function not found");

        func(args)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
