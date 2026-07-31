#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<ImportNode>,
    pub plugins: Vec<ImportNode>,
    pub globals: Vec<VariableNode>,
    pub classes: Vec<ClassNode>,
    pub functions: Vec<FunctionNode>,
    pub start: Option<FunctionNode>,
    pub update: Option<FunctionNode>,
    /// Обработчики событий вида `@cons.imput(msgg){...}`, которые
    /// ядро/плагин может вызвать по имени события (namespace + event).
    pub event_handlers: Vec<EventHandlerNode>,
}

/// `@cons.imput(msgg){...}` ->
/// EventHandlerNode { namespace: "cons", event: "imput", param: Some("msgg"), body }
#[derive(Debug, Clone)]
pub struct EventHandlerNode {
    pub namespace: String,
    pub event: String,
    pub param: Option<String>,
    pub body: Vec<Statement>,
}

/// `i:system,drive2d;` -> ImportNode { names: ["system", "drive2d"] }
/// `a:console;`        -> ImportNode { names: ["console"] } (в Program.plugins)
#[derive(Debug, Clone)]
pub struct ImportNode {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeNode {
    Int,
    Float,
    Bool,
    String,
    /// "Число, близкое к бесконечности"
    Ncti,
    /// Объект с типизированными полями
    Json,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub var_type: TypeNode,
    pub is_array: bool,
    pub name: String,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ClassNode {
    pub name: String,
    pub variables: Vec<VariableNode>,
    pub functions: Vec<FunctionNode>,
}

/// ==========================================
/// Литералы
/// ==========================================

#[derive(Debug, Clone)]
pub enum ValueNode {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// ==========================================
/// Выражения
/// ==========================================

#[derive(Debug, Clone)]
pub enum Expression {
    /// Значение
    Value(ValueNode),

    /// Переменная (или имя функции $f, используемое как ссылка)
    Variable(String),

    /// Доступ к полю объекта
    Member {
        object: Box<Expression>,
        member: String,
    },

    /// Вызов функции: `object.function(arguments)` либо `function(arguments)`
    Call {
        object: Option<Box<Expression>>,
        function: String,
        arguments: Vec<Expression>,
    },

    /// Создание массива
    Array(Vec<Expression>),

    /// Индекс массива
    ArrayIndex {
        array: Box<Expression>,
        index: Box<Expression>,
    },

    /// Бинарное выражение
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    /// Унарное выражение
    Unary {
        operator: UnaryOperator,
        value: Box<Expression>,
    },

    /// `{int#"n":1, bool#"b":true, ...}` — литерал json-объекта
    JsonLiteral(Vec<(TypeNode, String, Expression)>),

    /// `выражение#тип` — привести значение к другому типу
    Cast {
        value: Box<Expression>,
        target_type: TypeNode,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    // Арифметика
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,

    // Сравнение
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    NotGreater,
    NotLess,
    NotGreaterEqual,
    NotLessEqual,

    // Логика
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    /// `?выражение` — проверка на истинность (в отличие от `!`, не инвертирует)
    Truthy,
    Negative,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Variable(VariableNode),

    /// Вложенное объявление функции внутри тела другой функции: `$v(){ $px(){} }`
    FunctionDef(FunctionNode),

    Assignment {
        target: Expression,
        value: Expression,
    },

    Expression(Expression),

    Return(Option<Expression>),

    If {
        condition: Expression,
        body: Vec<Statement>,
        /// Список ветвей `!?(cond){...}` в порядке следования
        else_if: Vec<(Expression, Vec<Statement>)>,
        else_body: Vec<Statement>,
    },

    While {
        condition: Expression,
        body: Vec<Statement>,
    },

    For {
        init: Box<Statement>,
        condition: Expression,
        step: Box<Statement>,
        body: Vec<Statement>,
    },

    Break,
    Continue,

    /// Немедленный выход из всего скрипта (`exets;`)
    Exit,

    /// `имя[.поле].#тип = значение;` — сменить тип переменной или поля
    /// json-объекта (в отличие от обычного присваивания, тип проверять не нужно).
    Retype {
        base: String,
        field: Option<String>,
        new_type: TypeNode,
        value: Expression,
    },

    /// `имя.#new = тип#"ключ":значение;` — добавить новое поле в json-объект
    AddJsonField {
        base: String,
        field_type: TypeNode,
        key: String,
        value: Expression,
    },
}
