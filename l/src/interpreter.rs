use std::collections::HashMap;

use crate::ast::*;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{JsonField, JsonObject, Value, ValueType};

enum Flow {
    Normal,
    Return(Value),
    Break,
    Continue,
}

pub struct Interpreter {
    runtime: Runtime,
    scope: Scope,
    functions: HashMap<String, FunctionNode>,
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
            return;
        }

        loop {
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

            Statement::Exit => std::process::exit(0),

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
                                        declared_type: Some(type_node_to_value_type(new_type)),
                                        value: new_value,
                                    },
                                );
                                self.scope.set(base, Value::Json(obj));
                            }
                            _ => panic!("'{}' is not a json object", base),
                        }
                    }
                }

                Flow::Normal
            }

            Statement::AddJsonField { base, field_type, key, value } => {
                let raw = self.eval(value);
                let (declared_type, new_value) = match field_type {
                    Some(t) => (Some(type_node_to_value_type(t)), cast_value(raw, t)),
                    None => (None, raw),
                };

                let current = self.scope.get(base).unwrap_or(Value::Null);

                match current {
                    Value::Json(mut obj) => {
                        obj.fields.insert(
                            key.clone(),
                            JsonField { declared_type, value: new_value },
                        );
                        self.scope.set(base, Value::Json(obj));
                    }
                    _ => panic!("'{}' is not a json object", base),
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

            Statement::CreateEntry { path, accessor, value } => {
                let new_value = self.eval(value);

                match accessor {
                    PathAccessor::Key(key) => {
                        let key = key.clone();
                        self.mutate_path(path, &mut |container| match container {
                            Value::Json(obj) => {
                                obj.fields.insert(
                                    key.clone(),
                                    JsonField { declared_type: None, value: new_value.clone() },
                                );
                            }
                            _ => panic!(
                                "Cannot create field '{}': target is not a json object",
                                key
                            ),
                        });
                    }
                    PathAccessor::Index(index_expr) => {
                        let idx = self.eval(index_expr).as_int();
                        if idx < 0 {
                            panic!("Array index cannot be negative");
                        }
                        let idx = idx as usize;
                        self.mutate_path(path, &mut |container| match container {
                            Value::Array(items) => {
                                while items.len() <= idx {
                                    items.push(Value::Null);
                                }
                                items[idx] = new_value.clone();
                            }
                            _ => panic!(
                                "Cannot create index {}: target is not an array",
                                idx
                            ),
                        });
                    }
                }

                Flow::Normal
            }

            Statement::DeleteEntry { path, accessor } => {
                match accessor {
                    PathAccessor::Key(key) => {
                        let key = key.clone();
                        self.mutate_path(path, &mut |container| match container {
                            Value::Json(obj) => {
                                if obj.fields.remove(&key).is_none() {
                                    panic!("Object has no field '{}' to delete", key);
                                }
                            }
                            _ => panic!(
                                "Cannot delete field '{}': target is not a json object",
                                key
                            ),
                        });
                    }
                    PathAccessor::Index(index_expr) => {
                        let idx = self.eval(index_expr).as_int();
                        if idx < 0 {
                            panic!("Array index cannot be negative");
                        }
                        let idx = idx as usize;
                        self.mutate_path(path, &mut |container| match container {
                            Value::Array(items) => {
                                if idx >= items.len() {
                                    panic!(
                                        "Array index {} out of bounds (len {})",
                                        idx,
                                        items.len()
                                    );
                                }
                                items.remove(idx);
                            }
                            _ => panic!(
                                "Cannot delete index {}: target is not an array",
                                idx
                            ),
                        });
                    }
                }

                Flow::Normal
            }

            Statement::TryCatch { try_body, error_var, catch_body } => {
                let scope_depth = self.scope.depth();

                let previous_hook = std::panic::take_hook();
                std::panic::set_hook(Box::new(|_| {}));
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    self.exec_block(try_body)
                }));
                std::panic::set_hook(previous_hook);

                match result {
                    Ok(flow) => flow,
                    Err(payload) => {
                        self.scope.truncate(scope_depth);

                        let message = if let Some(s) = payload.downcast_ref::<String>() {
                            s.clone()
                        } else if let Some(s) = payload.downcast_ref::<&str>() {
                            s.to_string()
                        } else {
                            "unknown error".to_string()
                        };

                        self.scope.push();
                        self.scope.declare(error_var, Value::String(message));
                        let flow = self.exec_statements(catch_body);
                        self.scope.pop();
                        flow
                    }
                }
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

            Expression::Member { object, member } => {
                let member = member.clone();
                self.mutate_path(object, &mut |container| match container {
                    Value::Json(obj) => {
                        let existing = obj.fields.get(&member).cloned().unwrap_or_else(|| {
                            panic!(
                                "Object has no field '{}'. Use .#new.{} = value to create it.",
                                member, member
                            )
                        });

                        if let Some(declared) = &existing.declared_type {
                            if value.get_type() != *declared {
                                panic!(
                                    "Cannot assign a value of type {:?} to field '{}' (declared type {:?}). \
                                     Use #type to change its type.",
                                    value.get_type(), member, declared
                                );
                            }
                        }

                        obj.fields.insert(
                            member.clone(),
                            JsonField { declared_type: existing.declared_type, value: value.clone() },
                        );
                    }
                    _ => panic!("'{}' is not a json object", member),
                });
            }

            Expression::ArrayIndex { array, index } => {
                let idx = self.eval(index).as_int();
                if idx < 0 {
                    panic!("Array index cannot be negative");
                }
                let idx = idx as usize;

                self.mutate_path(array, &mut |container| match container {
                    Value::Array(items) => {
                        if idx >= items.len() {
                            panic!(
                                "Array index {} out of bounds (len {}). Use .#new[{}] = value to create it.",
                                idx, items.len(), idx
                            );
                        }
                        items[idx] = value.clone();
                    }
                    _ => panic!("Indexing is only supported for arrays"),
                });
            }

            _ => panic!(
                "Assignment is only supported for plain variables, json object fields, and array elements"
            ),
        }
    }

    /// Разрешает `expr` (переменную с произвольной цепочкой `.field` / `[index]`)
    /// до значения-контейнера и применяет к нему `f`, а затем сохраняет
    /// результат обратно в scope. Используется для `#new` / `#null`, а также
    /// для обычного присваивания в поля и элементы массива.
    ///
    /// Навигация по существующему пути (сами `.field` / `[index]` внутри
    /// `expr`) ничего не создаёт автоматически — отсутствующее поле или
    /// индекс вне границ вызывает панику. Создание/удаление конечного
    /// элемента — это то, что делает переданное `f`.
    fn mutate_path(&mut self, expr: &Expression, f: &mut dyn FnMut(&mut Value)) {
        match expr {
            Expression::Variable(name) => {
                let mut v = self.scope.get(name).unwrap_or(Value::Null);
                f(&mut v);
                self.scope.set(name, v);
            }

            Expression::Member { object, member } => {
                let member = member.clone();
                self.mutate_path(object, &mut |container: &mut Value| match container {
                    Value::Json(obj) => {
                        let field = obj.fields.entry(member.clone()).or_insert_with(|| JsonField {
                            declared_type: None,
                            value: Value::Null,
                        });
                        f(&mut field.value);
                    }
                    _ => panic!("'{}' is not a json object", member),
                });
            }

            Expression::ArrayIndex { array, index } => {
                let idx = self.eval(index).as_int();
                if idx < 0 {
                    panic!("Array index cannot be negative");
                }
                let idx = idx as usize;

                self.mutate_path(array, &mut |container: &mut Value| match container {
                    Value::Array(items) => {
                        if idx >= items.len() {
                            panic!("Array index {} out of bounds (len {})", idx, items.len());
                        }
                        f(&mut items[idx]);
                    }
                    _ => panic!("Indexing is only supported for arrays"),
                });
            }

            _ => panic!("This expression cannot be used as an assignment/mutation target"),
        }
    }


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
                    _ => panic!("Indexing is only supported for arrays"),
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
                    _ => panic!("This value has no field '{}'", member),
                }
            }

            Expression::Unary { operator, value } => {
                let value = self.eval(value);
                match operator {
                    UnaryOperator::Not => Value::Bool(!value.as_bool()),
                    UnaryOperator::Truthy => Value::Bool(value.as_bool()),
                    UnaryOperator::Negative => match value {
                        Value::Float(v) => Value::Float(-v),
                        Value::Ncti(_) => panic!("Negative ncti numbers are not supported yet"),
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
                    let (declared_type, value) = match field_type {
                        Some(t) => (Some(type_node_to_value_type(t)), cast_value(raw, t)),
                        None => (None, raw),
                    };
                    fields.insert(key.clone(), JsonField { declared_type, value });
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
                    panic!("Division for ncti is not supported yet")
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

        if let Some(Expression::Variable(namespace)) = object {
            if self.runtime.namespaces.contains_key(namespace) {
                return self.runtime.call_native(namespace, function, args);
            }

            let key = format!("{}.{}", namespace, function);
            if let Some(target) = self.functions.get(&key).cloned() {
                return self.call_user_function(&target, args);
            }
        }

        if let Some(target) = self.functions.get(function).cloned() {
            return self.call_user_function(&target, args);
        }

        panic!("Function '{}' not found", function);
    }

    fn call_user_function(&mut self, function: &FunctionNode, args: Vec<Value>) -> Value {
        if args.len() != function.params.len() {
            panic!(
                "Function '{}' expects {} argument(s), got {}",
                function.name,
                function.params.len(),
                args.len()
            );
        }

        self.scope.push();

        for ((param_type, param_name), arg_value) in function.params.iter().zip(args) {
            let value = match param_type {
                TypeNode::Ncti => Value::Ncti(arg_value.as_ncti()),
                TypeNode::Float => Value::Float(arg_value.as_float()),
                _ => arg_value,
            };
            self.scope.declare(param_name, value);
        }

        let flow = self.exec_statements(&function.body);
        self.scope.pop();

        match flow {
            Flow::Return(value) => value,
            _ => Value::Null,
        }
    }
}

fn cast_value(value: Value, target: &TypeNode) -> Value {
    match target {
        TypeNode::Int => match value {
            Value::String(s) => Value::Int(
                s.trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("Could not convert \"{}\" to int", s)),
            ),
            other => Value::Int(other.as_int()),
        },
        TypeNode::Float => match value {
            Value::String(s) => Value::Float(
                s.trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("Could not convert \"{}\" to float", s)),
            ),
            other => Value::Float(other.as_float()),
        },
        TypeNode::Bool => Value::Bool(value.as_bool()),
        TypeNode::String => Value::String(value.as_string()),
        TypeNode::Ncti => Value::Ncti(value.as_ncti()),
        TypeNode::Json => match value {
            Value::String(s) => {
                Value::Array(s.chars().map(|c| Value::String(c.to_string())).collect())
            }
            other => other,
        },
        TypeNode::Custom(_) => value,
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
