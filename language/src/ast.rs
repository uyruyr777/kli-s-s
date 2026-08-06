#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<ImportNode>,
    pub plugins: Vec<ImportNode>,
    pub globals: Vec<VariableNode>,
    pub classes: Vec<ClassNode>,
    pub functions: Vec<FunctionNode>,
    pub start: Option<FunctionNode>,
    pub update: Option<FunctionNode>,
    pub event_handlers: Vec<EventHandlerNode>,
}

#[derive(Debug, Clone)]
pub struct EventHandlerNode {
    pub namespace: String,
    pub event: String,
    pub param: Option<String>,
    pub body: Vec<Statement>,
}

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
    Ncti,
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
    pub params: Vec<(TypeNode, String)>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ClassNode {
    pub name: String,
    pub variables: Vec<VariableNode>,
    pub functions: Vec<FunctionNode>,
}

#[derive(Debug, Clone)]
pub enum ValueNode {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Expression {

    Value(ValueNode),

    Variable(String),

    Member {
        object: Box<Expression>,
        member: String,
    },

    Call {
        object: Option<Box<Expression>>,
        function: String,
        arguments: Vec<Expression>,
    },

    Array(Vec<Expression>),

    ArrayIndex {
        array: Box<Expression>,
        index: Box<Expression>,
    },

    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    Unary {
        operator: UnaryOperator,
        value: Box<Expression>,
    },

    JsonLiteral(Vec<(Option<TypeNode>, String, Expression)>),

    Cast {
        value: Box<Expression>,
        target_type: TypeNode,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
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
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Truthy,
    Negative,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Variable(VariableNode),

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

    Exit,

    Retype {
        base: String,
        field: Option<String>,
        new_type: TypeNode,
        value: Expression,
    },

    AddJsonField {
        base: String,
        field_type: Option<TypeNode>,
        key: String,
        value: Expression,
    },
}
