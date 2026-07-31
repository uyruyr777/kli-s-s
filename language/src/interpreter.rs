use std::collections::HashMap;

use crate::ast::*;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{JsonField, JsonObject, Value, ValueType};

/// Результат выполнения одного оператора: позволяет return/break/continue
/// "пробиться" наверх сквозь вложенные блоки (if/while/for).
enum Flow {
    Normal,
    Return(Value),
    Break,
    Continue,
}

pub struct Interpreter {
    runtime: Runtime,
    scope: Scope,
    /// Все именованные функции программы:
    /// `$f` хранится под ключом "f", вложенная `$px` внутри `$v` — под "v.px".
    functions: HashMap<String, FunctionNode>,
    /// Обработчики событий `@namespace.event(param){...}`, ключ — (namespace, event).
    event_handlers: HashMap<(String, String), EventHandlerNode>,
}

impl Interpreter {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            scope: Scope::new(),
            functions: HashMap::new(),
            event_handlers: HashMap::new(),
        }
    }

    /// Запустить программу:
    /// 1. глобальные переменные и функции регистрируются;
    /// 2. `@start` выполняется один раз — прямо в базовой области видимости,
    ///    поэтому переменные, объявленные внутри `@start`, ведут себя как
    ///    глобальные и видны в `@update` и обработчиках событий;
    /// 3. если есть `@update`, обработчик события или хотя бы одна активная
    ///    tick-функция (например, открытое окно) — запускается цикл: на
    ///    каждой итерации выполняются tick-функции (прокачка окон и т. п.),
    ///    опрашиваются источники событий, для которых есть обработчик, и
    ///    выполняется `@update`. Цикл прерывается вызовом `exets`
    ///    (Statement::Exit) или когда все tick-функции вернут false и нет
    ///    `@update`.
    pub fn run(&mut self, program: &Program) {
        for var in &program.globals {
            self.declare_variable(var);
        }

        for function in &program.functions {
            self.register_function(function, None);
        }

        for handler in &program.event_handlers {
            self.event_handlers.insert(
                (handler.namespace.clone(), handler.event.clone()),
                handler.clone(),
            );
        }

        if let Some(start) = &program.start {
            self.exec_statements(&start.body);
        }

        // Опрашиваем только те источники событий, на которые в скрипте
        // реально есть обработчик — так ядро само решает, что умеет
        // генерировать, а язык не завязан на конкретные имена вроде "cons".
        let active_sources: Vec<(String, String, crate::runtime::EventSourceFn)> = self
            .runtime
            .event_sources
            .iter()
            .filter(|(namespace, event, _)| {
                self.event_handlers.contains_key(&(namespace.clone(), event.clone()))
            })
            .cloned()
            .collect();

        let tick_functions = self.runtime.tick_functions.clone();

        if program.update.is_none() && active_sources.is_empty() && tick_functions.is_empty() {
            return; // разовый скрипт без цикла обновления/событий/окон
        }

        loop {
            // Безусловные тики (например, прокачка открытых окон).
            // Если ВСЕ они вернули false (окон нет и т. п.) и нет @update —
            // завершаем цикл, иначе программа "висела" бы вечно без причины.
            let mut any_alive = false;
            for tick in &tick_functions {
                if tick() {
                    any_alive = true;
                }
            }

            if !any_alive && program.update.is_none() && active_sources.is_empty() {
                break;
            }

            for (namespace, event, poll) in &active_sources {
                if let Some(value) = poll() {
                    self.dispatch_event(namespace, event, value);
                }
            }

            if let Some(update) = &program.update {
                self.exec_statements(&update.body);
            }
        }
    }

    /// Вызвать обработчик `@namespace.event(param){...}`, если он объявлен.
    /// Параметр (если есть в объявлении) получает переданное значение;
    /// присваивания уже существующим внешним переменным (как `msg = msgg;`)
    /// находят их через `Scope::set`, которая ищет во внешних областях.
    fn dispatch_event(&mut self, namespace: &str, event: &str, arg: Value) {
        let key = (namespace.to_string(), event.to_string());
        let handler = match self.event_handlers.get(&key).cloned() {
            Some(handler) => handler,
            None => return,
        };

        self.scope.push();
        if let Some(param) = &handler.param {
            self.scope.declare(param, arg);
        }
        self.exec_statements(&handler.body);
        self.scope.pop();
    }

    /// Регистрирует функцию и рекурсивно все вложенные `$px(){}` внутри неё
    /// под составным именем "родитель.имя" (для вызова вида `$v.px()`).
    fn register_function(&mut self, function: &FunctionNode, parent: Option<&str>) {
        let key = match parent {
            Some(parent_name) => format!("{}.{}", parent_name, function.name),
            None => function.name.clone(),
        };

        for statement in &function.body {
            if let Statement::FunctionDef(nested) = statement {
                self.register_function(nested, Some(&key));
            }
        }

        self.functions.insert(key, function.clone());
    }

    fn declare_variable(&mut self, var: &VariableNode) {
        let raw = match &var.value {
            Some(expr) => self.eval(expr),
            None => Value::Null,
        };

        // Приводим начальное значение к объявленному типу: `#ncti big 0;` —
        // 0 упаковывается в ncti (один лимб), а без начального значения
        // получаем ncti-ноль/float-ноль по умолчанию.
        let value = if var.is_array {
            raw
        } else {
            match &var.var_type {
                TypeNode::Ncti => Value::Ncti(raw.as_ncti()),
                TypeNode::Float => Value::Float(raw.as_float()),
                _ => raw,
            }
        };

        self.scope.declare(&var.name, value);
    }

    // ==========================================
    // Операторы
    // ==========================================

    fn exec_block(&mut self, statements: &[Statement]) -> Flow {
        self.scope.push();
        let flow = self.exec_statements(statements);
        self.scope.pop();
        flow
    }

    fn exec_statements(&mut self, statements: &[Statement]) -> Flow {
        for statement in statements {
            match self.exec_statement(statement) {
                Flow::Normal => {}
                other => return other,
            }
        }
        Flow::Normal
    }

    fn exec_statement(&mut self, statement: &Statement) -> Flow {
        match statement {
            Statement::Variable(var) => {
                self.declare_variable(var);
                Flow::Normal
            }

            // Уже зарегистрирована заранее в register_function — здесь делать нечего
            Statement::FunctionDef(_) => Flow::Normal,

            Statement::Assignment { target, value } => {
                let value = self.eval(value);
                self.assign(target, value);
                Flow::Normal
            }

            Statement::Expression(expr) => {
                self.eval(expr);
                Flow::Normal
            }

            Statement::Return(expr) => {
                let value = match expr {
                    Some(e) => self.eval(e),
                    None => Value::Null,
                };
                Flow::Return(value)
            }

            Statement::Break => Flow::Break,
            Statement::Continue => Flow::Continue,

            // Полный выход из скрипта — немедленно завершает процесс.
            Statement::Exit => std::process::exit(0),

            // `имя[.поле].#тип = значение;` — смена типа (в отличие от обычного
            // присваивания, проверка совместимости типов не выполняется).
            Statement::Retype { base, field, new_type, value } => {
                let new_value = cast_value(self.eval(value), new_type);

                match field {
                    None => {
                        self.scope.set(base, new_value);
                    }
                    Some(field_name) => {
                        let current = self.scope.get(base).unwrap_or(Value::Null);
                        match current {
                            Value::Json(mut obj) => {
                                obj.fields.insert(
                                    field_name.clone(),
                                    JsonField {
                                        declared_type: type_node_to_value_type(new_type),
                                        value: new_value,
                                    },
                                );
                                self.scope.set(base, Value::Json(obj));
                            }
                            _ => panic!("'{}' не является json-объектом", base),
                        }
                    }
                }

                Flow::Normal
            }

            // `имя.#new = тип#"ключ":значение;` — добавить новое поле в json-объект
            Statement::AddJsonField { base, field_type, key, value } => {
                let new_value = cast_value(self.eval(value), field_type);
                let current = self.scope.get(base).unwrap_or(Value::Null);

                match current {
                    Value::Json(mut obj) => {
                        obj.fields.insert(
                            key.clone(),
                            JsonField {
                                declared_type: type_node_to_value_type(field_type),
                                value: new_value,
                            },
                        );
                        self.scope.set(base, Value::Json(obj));
                    }
                    _ => panic!("'{}' не является json-объектом", base),
                }

                Flow::Normal
            }

            Statement::If { condition, body, else_if, else_body } => {
                if self.eval(condition).as_bool() {
                    return self.exec_block(body);
                }

                for (elseif_condition, elseif_body) in else_if {
                    if self.eval(elseif_condition).as_bool() {
                        return self.exec_block(elseif_body);
                    }
                }

                if !else_body.is_empty() {
                    return self.exec_block(else_body);
                }

                Flow::Normal
            }

            Statement::While { condition, body } => {
                while self.eval(condition).as_bool() {
                    match self.exec_block(body) {
                        Flow::Break => break,
                        Flow::Continue | Flow::Normal => {}
                        returned @ Flow::Return(_) => return returned,
                    }
                }
                Flow::Normal
            }

            Statement::For { init, condition, step, body } => {
                self.scope.push();
                self.exec_statement(init);

                loop {
                    if !self.eval(condition).as_bool() {
                        break;
                    }

                    match self.exec_block(body) {
                        Flow::Break => break,
                        Flow::Continue | Flow::Normal => {}
                        returned @ Flow::Return(_) => {
                            self.scope.pop();
                            return returned;
                        }
                    }

                    self.exec_statement(step);
                }

                self.scope.pop();
                Flow::Normal
            }
        }
    }

    fn assign(&mut self, target: &Expression, value: Value) {
        match target {
            Expression::Variable(name) => self.scope.set(name, value),

            // `gg.number = "str";` — обычное присваивание полю json-объекта.
            // В отличие от `.#тип = значение`, тип поля здесь должен совпадать.
            Expression::Member { object, member } => {
                let base = match object.as_ref() {
                    Expression::Variable(name) => name.clone(),
                    _ => panic!(
                        "Присваивание полю поддерживается только напрямую у переменной (obj.field = ...)"
                    ),
                };

                let current = self.scope.get(&base).unwrap_or(Value::Null);
                match current {
                    Value::Json(mut obj) => {
                        let existing = obj.fields.get(member).cloned().unwrap_or_else(|| {
                            panic!("У объекта '{}' нет поля '{}'", base, member)
                        });

                        if value.get_type() != existing.declared_type {
                            panic!(
                                "Нельзя присвоить значение типа {:?} полю '{}' (тип {:?}). \
                                 Используйте {}.{}.#тип = значение, чтобы сменить тип поля.",
                                value.get_type(), member, existing.declared_type, base, member
                            );
                        }

                        obj.fields.insert(
                            member.clone(),
                            JsonField { declared_type: existing.declared_type, value },
                        );
                        self.scope.set(&base, Value::Json(obj));
                    }
                    _ => panic!("'{}' не является json-объектом", base),
                }
            }

            _ => panic!("Присваивание поддерживается только для простых переменных и полей json-объектов"),
        }
    }

    // ==========================================
    // Выражения
    // ==========================================

    fn eval(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::Value(ValueNode::Int(n)) => Value::Int(*n),
            Expression::Value(ValueNode::Float(n)) => Value::Float(*n),
            Expression::Value(ValueNode::Bool(b)) => Value::Bool(*b),
            Expression::Value(ValueNode::String(s)) => Value::String(s.clone()),

            Expression::Variable(name) => self.scope.get(name).unwrap_or(Value::Null),

            Expression::Array(items) => {
                Value::Array(items.iter().map(|item| self.eval(item)).collect())
            }

            Expression::ArrayIndex { array, index } => {
                let array = self.eval(array);
                let index = self.eval(index).as_int();
                match array {
                    Value::Array(items) => items.get(index as usize).cloned().unwrap_or(Value::Null),
                    _ => panic!("Индексирование доступно только для массивов"),
                }
            }

            Expression::Member { object, member } => {
                let obj = self.eval(object);
                match obj {
                    Value::Json(json) => json
                        .fields
                        .get(member)
                        .map(|f| f.value.clone())
                        .unwrap_or(Value::Null),
                    _ => panic!("У значения нет поля '{}'", member),
                }
            }

            Expression::Unary { operator, value } => {
                let value = self.eval(value);
                match operator {
                    UnaryOperator::Not => Value::Bool(!value.as_bool()),
                    UnaryOperator::Truthy => Value::Bool(value.as_bool()),
                    UnaryOperator::Negative => match value {
                        Value::Float(v) => Value::Float(-v),
                        Value::Ncti(_) => panic!("Отрицательные ncti-числа пока не поддерживаются"),
                        _ => Value::Int(-value.as_int()),
                    },
                }
            }

            Expression::Binary { left, operator, right } => self.eval_binary(left, operator, right),

            Expression::Call { object, function, arguments } => {
                self.eval_call(object.as_deref(), function, arguments)
            }

            Expression::JsonLiteral(entries) => {
                let mut fields = HashMap::new();
                for (field_type, key, value_expr) in entries {
                    let raw = self.eval(value_expr);
                    let value = cast_value(raw, field_type);
                    fields.insert(
                        key.clone(),
                        JsonField { declared_type: type_node_to_value_type(field_type), value },
                    );
                }
                Value::Json(JsonObject { fields })
            }

            Expression::Cast { value, target_type } => {
                let raw = self.eval(value);
                cast_value(raw, target_type)
            }
        }
    }

    fn eval_binary(&mut self, left: &Expression, operator: &BinaryOperator, right: &Expression) -> Value {
        let left = self.eval(left);
        let right = self.eval(right);

        let is_ncti = matches!(left, Value::Ncti(_)) || matches!(right, Value::Ncti(_));
        let is_float = matches!(left, Value::Float(_)) || matches!(right, Value::Float(_));

        match operator {
            BinaryOperator::Add => match (&left, &right) {
                (Value::String(_), _) | (_, Value::String(_)) => {
                    Value::String(format!("{}{}", left, right))
                }
                _ if is_ncti => Value::Ncti(left.as_ncti().add(&right.as_ncti())),
                _ if is_float => Value::Float(left.as_float() + right.as_float()),
                _ => Value::Int(left.as_int() + right.as_int()),
            },
            BinaryOperator::Subtract => {
                if is_ncti {
                    Value::Ncti(left.as_ncti().sub(&right.as_ncti()))
                } else if is_float {
                    Value::Float(left.as_float() - right.as_float())
                } else {
                    Value::Int(left.as_int() - right.as_int())
                }
            }
            BinaryOperator::Multiply => {
                if is_ncti {
                    Value::Ncti(left.as_ncti().mul(&right.as_ncti()))
                } else if is_float {
                    Value::Float(left.as_float() * right.as_float())
                } else {
                    Value::Int(left.as_int() * right.as_int())
                }
            }
            BinaryOperator::Divide => {
                if is_ncti {
                    panic!("Деление ncti пока не поддерживается")
                } else if is_float {
                    Value::Float(left.as_float() / right.as_float())
                } else {
                    Value::Int(left.as_int() / right.as_int())
                }
            }
            BinaryOperator::Mod => Value::Int(left.as_int() % right.as_int()),

            BinaryOperator::Equal => Value::Bool(self.values_equal(&left, &right, is_ncti, is_float)),
            BinaryOperator::NotEqual => Value::Bool(!self.values_equal(&left, &right, is_ncti, is_float)),
            BinaryOperator::Greater => Value::Bool(self.compare(&left, &right, is_ncti, is_float) == std::cmp::Ordering::Greater),
            BinaryOperator::Less => Value::Bool(self.compare(&left, &right, is_ncti, is_float) == std::cmp::Ordering::Less),
            BinaryOperator::GreaterEqual => Value::Bool(self.compare(&left, &right, is_ncti, is_float) != std::cmp::Ordering::Less),
            BinaryOperator::LessEqual => Value::Bool(self.compare(&left, &right, is_ncti, is_float) != std::cmp::Ordering::Greater),
            BinaryOperator::NotGreater => Value::Bool(self.compare(&left, &right, is_ncti, is_float) != std::cmp::Ordering::Greater),
            BinaryOperator::NotLess => Value::Bool(self.compare(&left, &right, is_ncti, is_float) != std::cmp::Ordering::Less),
            BinaryOperator::NotGreaterEqual => Value::Bool(self.compare(&left, &right, is_ncti, is_float) == std::cmp::Ordering::Less),
            BinaryOperator::NotLessEqual => Value::Bool(self.compare(&left, &right, is_ncti, is_float) == std::cmp::Ordering::Greater),

            BinaryOperator::And => Value::Bool(left.as_bool() && right.as_bool()),
            BinaryOperator::Or => Value::Bool(left.as_bool() || right.as_bool()),
        }
    }

    fn compare(&self, left: &Value, right: &Value, is_ncti: bool, is_float: bool) -> std::cmp::Ordering {
        if is_ncti {
            left.as_ncti().cmp(&right.as_ncti())
        } else if is_float {
            left.as_float()
                .partial_cmp(&right.as_float())
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            left.as_int().cmp(&right.as_int())
        }
    }

    fn values_equal(&self, left: &Value, right: &Value, is_ncti: bool, is_float: bool) -> bool {
        if is_ncti || is_float {
            self.compare(left, right, is_ncti, is_float) == std::cmp::Ordering::Equal
        } else {
            left == right
        }
    }

    fn eval_call(&mut self, object: Option<&Expression>, function: &str, arguments: &[Expression]) -> Value {
        let args: Vec<Value> = arguments.iter().map(|arg| self.eval(arg)).collect();

        // `namespace.function(args)` — ядро/плагин, зарегистрированный в Runtime
        if let Some(Expression::Variable(namespace)) = object {
            if self.runtime.namespaces.contains_key(namespace) {
                return self.runtime.call_native(namespace, function, args);
            }

            // `$v.px()` — вызов вложенной функции по составному имени "v.px"
            let key = format!("{}.{}", namespace, function);
            if let Some(target) = self.functions.get(&key).cloned() {
                return self.call_user_function(&target);
            }
        }

        // Обычный вызов пользовательской функции `$f()`
        if let Some(target) = self.functions.get(function).cloned() {
            return self.call_user_function(&target);
        }

        panic!("Функция '{}' не найдена", function);
    }

    fn call_user_function(&mut self, function: &FunctionNode) -> Value {
        self.scope.push();
        let flow = self.exec_statements(&function.body);
        self.scope.pop();

        match flow {
            Flow::Return(value) => value,
            _ => Value::Null,
        }
    }
}

/// Приводит значение к другому типу — используется и для `выражение#тип`,
/// и для `.#тип = значение`, и при построении json-литералов/добавлении полей.
/// Строки при преобразовании в int/float реально парсятся (`"11"#int` -> 11),
/// а не просто отбрасываются, как при обычном `as_int`/`as_float`.
fn cast_value(value: Value, target: &TypeNode) -> Value {
    match target {
        TypeNode::Int => match value {
            Value::String(s) => Value::Int(
                s.trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("Не удалось преобразовать \"{}\" в int", s)),
            ),
            other => Value::Int(other.as_int()),
        },
        TypeNode::Float => match value {
            Value::String(s) => Value::Float(
                s.trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("Не удалось преобразовать \"{}\" в float", s)),
            ),
            other => Value::Float(other.as_float()),
        },
        TypeNode::Bool => Value::Bool(value.as_bool()),
        TypeNode::String => Value::String(value.as_string()),
        TypeNode::Ncti => Value::Ncti(value.as_ncti()),
        // json/классы как цель приведения пока не поддержаны — возвращаем как есть
        TypeNode::Json | TypeNode::Custom(_) => value,
    }
}

fn type_node_to_value_type(t: &TypeNode) -> ValueType {
    match t {
        TypeNode::Int => ValueType::Int,
        TypeNode::Float => ValueType::Float,
        TypeNode::Bool => ValueType::Bool,
        TypeNode::String => ValueType::String,
        TypeNode::Ncti => ValueType::Ncti,
        TypeNode::Json => ValueType::Json,
        TypeNode::Custom(_) => ValueType::Null,
    }
}
