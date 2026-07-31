use std::collections::HashMap;

use crate::value::Value;

pub type NativeFunction = fn(Vec<Value>) -> Value;

/// Функция-источник события: ядро/плагин её опрашивает интерпретатор на
/// каждой итерации цикла. Возвращает `Some(значение)`, если событие
/// произошло (например, была введена строка в консоль), иначе `None`.
pub type EventSourceFn = fn() -> Option<Value>;

/// Функция, которую интерпретатор вызывает на каждой итерации цикла
/// БЕЗУСЛОВНО (не только когда есть подходящий обработчик в скрипте) —
/// нужна, например, чтобы "прокачивать" открытые окна (обрабатывать их
/// системные события, перерисовывать). Возвращает true, если ядро всё ещё
/// "живо" (например, есть хотя бы одно открытое окно) — тогда цикл
/// интерпретатора не завершится, даже если в скрипте нет `@update`.
pub type TickFn = fn() -> bool;

/// Пространство имён — ядро (system, drive2d, ...) или плагин (console, ...)
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

    /// Цепочечная регистрация: `namespace.function("print", f).function("read", g)`
    pub fn function(&mut self, name: &str, function: NativeFunction) -> &mut Self {
        self.functions.insert(name.to_string(), function);
        self
    }
}

/// Исполнитель программы: хранит зарегистрированные ядра/плагины
/// и умеет вызывать их нативные функции по имени.
pub struct Runtime {
    pub namespaces: HashMap<String, Namespace>,
    /// Источники событий, зарегистрированные ядрами/плагинами:
    /// (namespace, event, функция-опрос). Интерпретатор сам решает,
    /// какие из них реально опрашивать — только те, на которые в
    /// скрипте есть обработчик `@namespace.event(...){...}`.
    pub event_sources: Vec<(String, String, EventSourceFn)>,
    /// Функции, вызываемые интерпретатором каждую итерацию безусловно
    /// (см. `TickFn`) — например, "прокачка" открытых окон.
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

    /// Зарегистрировать источник события: `runtime.event_source("cons", "imput", poll_fn);`
    pub fn event_source(&mut self, namespace: &str, event: &str, poll: EventSourceFn) {
        self.event_sources.push((namespace.to_string(), event.to_string(), poll));
    }

    /// Зарегистрировать безусловную функцию тика: `runtime.register_tick(pump_windows);`
    pub fn register_tick(&mut self, tick: TickFn) {
        self.tick_functions.push(tick);
    }

    pub fn register_namespace(&mut self, namespace: Namespace) {
        self.namespaces.insert(namespace.name.clone(), namespace);
    }

    /// Получить (создав при необходимости) пространство имён по имени —
    /// используется в `kernel`/`plugin` модулях как:
    /// `runtime.namespace("cons").function("print", cons_print);`
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
