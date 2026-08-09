use crate::ast::*;
use crate::lexer::{DataType, Keyword, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.current().kind == *kind
    }

    fn consume(&mut self, kind: TokenKind) {
        if self.current().kind == kind {
            self.advance();
        } else {
            panic!(
                "Expected {:?}, found {:?} ({}:{})",
                kind,
                self.current().kind,
                self.current().line,
                self.current().column
            );
        }
    }

    fn expect_identifier(&mut self) -> String {
        match &self.current().kind {
            TokenKind::Identifier(name) => {
                let result = name.clone();
                self.advance();
                result
            }
            _ => panic!(
                "Expected an identifier, found {:?}",
                self.current().kind
            ),
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut program = Program {
            imports: Vec::new(),
            plugins: Vec::new(),
            globals: Vec::new(),
            classes: Vec::new(),
            functions: Vec::new(),
            start: None,
            update: None,
            event_handlers: Vec::new(),
        };

        while self.current().kind != TokenKind::EOF {
            match &self.current().kind {
                TokenKind::Keyword(Keyword::Import) => {
                    program.imports.push(self.parse_import_like());
                }

                TokenKind::Keyword(Keyword::Plugin) => {
                    program.plugins.push(self.parse_import_like());
                }

                TokenKind::Hash => {
                    program.globals.push(self.parse_variable());
                }

                TokenKind::Class(_) => {
                    program.classes.push(self.parse_class());
                }

                TokenKind::Function(_) => {
                    program.functions.push(self.parse_function());
                }

                TokenKind::At => {
                    self.advance();

                    match &self.current().kind {
                        TokenKind::Keyword(Keyword::Start) => {
                            program.start = Some(self.parse_special_function("start"));
                        }
                        TokenKind::Keyword(Keyword::Update) => {
                            program.update = Some(self.parse_special_function("update"));
                        }
                        TokenKind::Identifier(name) => {
                            let namespace = name.clone();
                            self.advance();
                            program.event_handlers.push(self.parse_event_handler(namespace));
                        }
                        _ => panic!("Unknown construct after @"),
                    }
                }

                _ => panic!("Unexpected token {:?}", self.current().kind),
            }
        }

        program
    }

    fn parse_import_like(&mut self) -> ImportNode {
        self.advance();
        self.consume(TokenKind::Colon);

        let mut names = vec![self.expect_identifier()];
        while self.check(&TokenKind::Comma) {
            self.advance();
            names.push(self.expect_identifier());
        }

        self.consume(TokenKind::Semicolon);
        ImportNode { names }
    }

    fn parse_type(&mut self) -> TypeNode {
        let node = match &self.current().kind {
            TokenKind::Type(DataType::Int) => TypeNode::Int,
            TokenKind::Type(DataType::Float) => TypeNode::Float,
            TokenKind::Type(DataType::Bool) => TypeNode::Bool,
            TokenKind::Type(DataType::String) => TypeNode::String,
            TokenKind::Type(DataType::Ncti) => TypeNode::Ncti,
            TokenKind::Type(DataType::Json) => TypeNode::Json,
            TokenKind::Identifier(name) => TypeNode::Custom(name.clone()),
            _ => panic!("Expected a type, found {:?}", self.current().kind),
        };
        self.advance();
        node
    }

    fn parse_variable(&mut self) -> VariableNode {
        self.consume(TokenKind::Hash);
        let var_type = self.parse_type();

        let is_array = if self.check(&TokenKind::OpenBracket) {
            self.advance();
            self.consume(TokenKind::CloseBracket);
            true
        } else {
            false
        };

        let name = self.expect_identifier();

        let value = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression())
        };

        self.consume(TokenKind::Semicolon);

        VariableNode { var_type, is_array, name, value }
    }

    fn parse_variable_no_semicolon(&mut self) -> VariableNode {
        self.consume(TokenKind::Hash);
        let var_type = self.parse_type();

        let is_array = if self.check(&TokenKind::OpenBracket) {
            self.advance();
            self.consume(TokenKind::CloseBracket);
            true
        } else {
            false
        };

        let name = self.expect_identifier();

        let value = Some(self.parse_expression());

        VariableNode { var_type, is_array, name, value }
    }

    fn parse_class(&mut self) -> ClassNode {
        let name = match &self.current().kind {
            TokenKind::Class(name) => name.clone(),
            _ => panic!("Expected a class name"),
        };
        self.advance();

        self.consume(TokenKind::OpenBrace);

        let mut variables = Vec::new();
        let mut functions = Vec::new();

        while !self.check(&TokenKind::CloseBrace) {
            match &self.current().kind {
                TokenKind::Hash => variables.push(self.parse_variable()),
                TokenKind::Function(_) => functions.push(self.parse_function()),
                _ => panic!(
                    "Unexpected token inside class: {:?}",
                    self.current().kind
                ),
            }
        }

        self.consume(TokenKind::CloseBrace);

        ClassNode { name, variables, functions }
    }

    fn parse_function(&mut self) -> FunctionNode {
        let name = match &self.current().kind {
            TokenKind::Function(name) => name.clone(),
            _ => panic!("Expected a function name"),
        };
        self.advance();

        let params = self.parse_params();
        let body = self.parse_block();

        FunctionNode { name, params, body }
    }

    fn parse_special_function(&mut self, name: &str) -> FunctionNode {
        self.advance();
        let body = self.parse_block();
        FunctionNode { name: name.to_string(), params: Vec::new(), body }
    }

    fn parse_params(&mut self) -> Vec<(TypeNode, String)> {
        self.consume(TokenKind::OpenParen);
        let mut params = Vec::new();

        if !self.check(&TokenKind::CloseParen) {
            loop {
                let param_type = self.parse_type();
                self.consume(TokenKind::Hash);
                let name = self.expect_identifier();
                params.push((param_type, name));

                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.consume(TokenKind::CloseParen);
        params
    }

    fn parens_followed_by_brace(&self) -> bool {
        let mut i = self.pos;
        let mut depth = 0i32;

        loop {
            match self.tokens.get(i).map(|t| &t.kind) {
                Some(TokenKind::OpenParen) => depth += 1,
                Some(TokenKind::CloseParen) => {
                    depth -= 1;
                    if depth == 0 {
                        return matches!(
                            self.tokens.get(i + 1).map(|t| &t.kind),
                            Some(TokenKind::OpenBrace)
                        );
                    }
                }
                Some(TokenKind::EOF) | None => return false,
                _ => {}
            }
            i += 1;
        }
    }

    fn parse_event_handler(&mut self, namespace: String) -> EventHandlerNode {
        self.consume(TokenKind::Dot);
        let event = self.expect_identifier();

        self.consume(TokenKind::OpenParen);
        let param = if !self.check(&TokenKind::CloseParen) {
            Some(self.expect_identifier())
        } else {
            None
        };
        self.consume(TokenKind::CloseParen);

        let body = self.parse_block();

        EventHandlerNode { namespace, event, param, body }
    }

    fn looks_like_retype_or_addfield(&self) -> bool {
        let mut i = self.pos + 1;

        let is_dot = |t: Option<&TokenKind>| matches!(t, Some(TokenKind::Dot));
        let is_identifier = |t: Option<&TokenKind>| matches!(t, Some(TokenKind::Identifier(_)));

        if is_dot(self.tokens.get(i).map(|t| &t.kind))
            && is_identifier(self.tokens.get(i + 1).map(|t| &t.kind))
            && is_dot(self.tokens.get(i + 2).map(|t| &t.kind))
        {
            i += 2;
        }

        if !(is_dot(self.tokens.get(i).map(|t| &t.kind))
            && matches!(self.tokens.get(i + 1).map(|t| &t.kind), Some(TokenKind::Hash)))
        {
            return false;
        }

        match self.tokens.get(i + 2).map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) if name == "null" => false,
            Some(TokenKind::Identifier(name)) if name == "new" => {
                matches!(self.tokens.get(i + 3).map(|t| &t.kind), Some(TokenKind::Assign))
            }
            _ => true,
        }
    }

    fn parse_retype_or_addfield_statement(&mut self) -> Statement {
        let base = self.expect_identifier();

        let mut field = None;

        if self.check(&TokenKind::Dot) {
            let save = self.pos;
            self.advance();

            if let TokenKind::Identifier(name) = self.current().kind.clone() {
                self.advance();
                if self.check(&TokenKind::Dot) {
                    field = Some(name);
                } else {
                    self.pos = save;
                }
            } else {
                self.pos = save;
            }
        }

        self.consume(TokenKind::Dot);
        self.consume(TokenKind::Hash);

        let is_new_field = matches!(&self.current().kind, TokenKind::Identifier(name) if name == "new");

        if is_new_field {
            self.advance();
            self.consume(TokenKind::Assign);
            let (field_type, key, value) = self.parse_json_field();
            self.consume(TokenKind::Semicolon);
            Statement::AddJsonField { base, field_type, key, value }
        } else {
            let new_type = self.parse_type();
            self.consume(TokenKind::Assign);
            let value = self.parse_expression();
            self.consume(TokenKind::Semicolon);
            Statement::Retype { base, field, new_type, value }
        }
    }

    fn parse_json_field(&mut self) -> (Option<TypeNode>, String, Expression) {
        let next_is_colon = matches!(
            self.tokens.get(self.pos + 1).map(|t| &t.kind),
            Some(TokenKind::Colon)
        );

        if next_is_colon {
            let key = match &self.current().kind {
                TokenKind::String(s) => s.clone(),
                other => panic!(
                    "Expected a string key, found {:?} ({}:{})",
                    other,
                    self.current().line,
                    self.current().column
                ),
            };
            self.advance();
            self.consume(TokenKind::Colon);
            let value = self.parse_expression();
            return (None, key, value);
        }

        let field_type = self.parse_type();
        self.consume(TokenKind::Hash);

        let key = match &self.current().kind {
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            other => panic!(
                "Expected a string key after '#', found {:?} ({}:{})",
                other,
                self.current().line,
                self.current().column
            ),
        };

        self.consume(TokenKind::Colon);
        let value = self.parse_expression();

        (Some(field_type), key, value)
    }

    fn parse_json_literal(&mut self) -> Expression {
        self.consume(TokenKind::OpenBrace);
        let mut fields = Vec::new();

        if !self.check(&TokenKind::CloseBrace) {
            loop {
                fields.push(self.parse_json_field());

                if self.check(&TokenKind::Comma) {
                    self.advance();
                    if self.check(&TokenKind::CloseBrace) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        self.consume(TokenKind::CloseBrace);
        Expression::JsonLiteral(fields)
    }

    fn parse_block(&mut self) -> Vec<Statement> {
        self.consume(TokenKind::OpenBrace);
        let mut statements = Vec::new();
        while !self.check(&TokenKind::CloseBrace) {
            statements.push(self.parse_statement());
        }
        self.consume(TokenKind::CloseBrace);
        statements
    }

    fn parse_statement(&mut self) -> Statement {
        match &self.current().kind {
            TokenKind::Hash => Statement::Variable(self.parse_variable()),

            TokenKind::Function(_)
                if matches!(
                    self.tokens.get(self.pos + 1).map(|t| &t.kind),
                    Some(TokenKind::OpenParen)
                ) =>
            {
                self.parse_function_statement()
            }

            TokenKind::Keyword(Keyword::Return) => {
                self.advance();
                let value = if self.check(&TokenKind::Semicolon) {
                    None
                } else {
                    Some(self.parse_expression())
                };
                self.consume(TokenKind::Semicolon);
                Statement::Return(value)
            }

            TokenKind::Keyword(Keyword::Break) => {
                self.advance();
                self.consume(TokenKind::Semicolon);
                Statement::Break
            }

            TokenKind::Keyword(Keyword::Continue) => {
                self.advance();
                self.consume(TokenKind::Semicolon);
                Statement::Continue
            }

            TokenKind::Keyword(Keyword::Exit) => {
                self.advance();
                self.consume(TokenKind::Semicolon);
                Statement::Exit
            }

            TokenKind::Question => self.parse_if(),

            TokenKind::At => self.parse_loop(),

            TokenKind::Percent => self.parse_try_catch(),

            TokenKind::Identifier(_) if self.looks_like_retype_or_addfield() => {
                self.parse_retype_or_addfield_statement()
            }

            TokenKind::Identifier(name)
                if matches!(
                    self.tokens.get(self.pos + 1).map(|t| &t.kind),
                    Some(TokenKind::Plus)
                        | Some(TokenKind::Minus)
                        | Some(TokenKind::Star)
                        | Some(TokenKind::Slash)
                        | Some(TokenKind::Percent)
                ) =>
            {
                let name = name.clone();
                self.advance();

                let operator = match &self.current().kind {
                    TokenKind::Plus => BinaryOperator::Add,
                    TokenKind::Minus => BinaryOperator::Subtract,
                    TokenKind::Star => BinaryOperator::Multiply,
                    TokenKind::Slash => BinaryOperator::Divide,
                    TokenKind::Percent => BinaryOperator::Mod,
                    _ => unreachable!(),
                };
                self.advance();

                let right = self.parse_expression();
                self.consume(TokenKind::Semicolon);

                let value = Expression::Binary {
                    left: Box::new(Expression::Variable(name.clone())),
                    operator,
                    right: Box::new(right),
                };

                Statement::Assignment { target: Expression::Variable(name), value }
            }

            _ => {
                let expr = self.parse_expression();

                if self.check(&TokenKind::Dot)
                    && matches!(
                        self.tokens.get(self.pos + 1).map(|t| &t.kind),
                        Some(TokenKind::Hash)
                    )
                {
                    return self.parse_mutation_statement(expr);
                }

                if self.check(&TokenKind::Assign) {
                    self.advance();
                    let value = self.parse_expression();
                    self.consume(TokenKind::Semicolon);
                    Statement::Assignment { target: expr, value }
                } else {
                    self.consume(TokenKind::Semicolon);
                    Statement::Expression(expr)
                }
            }
        }
    }

    fn parse_mutation_statement(&mut self, path: Expression) -> Statement {
        self.consume(TokenKind::Dot);
        self.consume(TokenKind::Hash);

        let is_new = match &self.current().kind {
            TokenKind::Identifier(name) if name == "new" => true,
            TokenKind::Identifier(name) if name == "null" => false,
            other => panic!("Expected 'new' or 'null' after '.#', found {:?}", other),
        };
        self.advance();

        let accessor = self.parse_path_accessor();

        if is_new {
            self.consume(TokenKind::Assign);
            let value = self.parse_expression();
            self.consume(TokenKind::Semicolon);
            Statement::CreateEntry { path, accessor, value }
        } else {
            self.consume(TokenKind::Semicolon);
            Statement::DeleteEntry { path, accessor }
        }
    }

    fn parse_path_accessor(&mut self) -> PathAccessor {
        match &self.current().kind {
            TokenKind::Dot => {
                self.advance();
                let key = match &self.current().kind {
                    TokenKind::Identifier(name) => {
                        let n = name.clone();
                        self.advance();
                        n
                    }
                    TokenKind::String(text) => {
                        let n = text.clone();
                        self.advance();
                        n
                    }
                    other => panic!("Expected a field name after '.', found {:?}", other),
                };
                PathAccessor::Key(key)
            }
            TokenKind::OpenBracket => {
                self.advance();
                let index = self.parse_expression();
                self.consume(TokenKind::CloseBracket);
                PathAccessor::Index(index)
            }
            other => panic!(
                "Expected '.key' or '[index]' after '#new'/'#null', found {:?}",
                other
            ),
        }
    }

    fn parse_function_statement(&mut self) -> Statement {
        let name = match &self.current().kind {
            TokenKind::Function(name) => name.clone(),
            _ => unreachable!(),
        };
        self.advance();

        if self.parens_followed_by_brace() {
            let params = self.parse_params();
            let body = self.parse_block();
            Statement::FunctionDef(FunctionNode { name, params, body })
        } else {
            let arguments = self.parse_arguments();
            self.consume(TokenKind::Semicolon);
            Statement::Expression(Expression::Call {
                object: None,
                function: name,
                arguments,
            })
        }
    }

    fn parse_if(&mut self) -> Statement {
        self.consume(TokenKind::Question);
        self.consume(TokenKind::OpenParen);
        let condition = self.parse_expression();
        self.consume(TokenKind::CloseParen);
        let body = self.parse_block();

        let mut else_if = Vec::new();
        let mut else_body = Vec::new();

        while self.check(&TokenKind::Exclamation) {
            let save = self.pos;
            self.advance();

            if self.check(&TokenKind::Question) {
                self.advance();
                self.consume(TokenKind::OpenParen);
                let elseif_condition = self.parse_expression();
                self.consume(TokenKind::CloseParen);
                let elseif_body = self.parse_block();
                else_if.push((elseif_condition, elseif_body));
            } else if self.check(&TokenKind::OpenBrace) {
                else_body = self.parse_block();
                break;
            } else {
                self.pos = save;
                break;
            }
        }

        Statement::If { condition, body, else_if, else_body }
    }

    fn parse_try_catch(&mut self) -> Statement {
        self.consume(TokenKind::Percent);
        let try_body = self.parse_block();

        match &self.current().kind {
            TokenKind::Identifier(name) if name == "e" => {}
            other => panic!(
                "Expected 'e(var)' after try block, found {:?} ({}:{})",
                other,
                self.current().line,
                self.current().column
            ),
        }
        self.advance();

        self.consume(TokenKind::OpenParen);
        let error_var = self.expect_identifier();
        self.consume(TokenKind::CloseParen);

        let catch_body = self.parse_block();

        Statement::TryCatch { try_body, error_var, catch_body }
    }

    fn parse_loop(&mut self) -> Statement {
        self.consume(TokenKind::At);

        match &self.current().kind {
            TokenKind::Keyword(Keyword::While) => {
                self.advance();
                self.consume(TokenKind::OpenParen);
                let condition = self.parse_expression();
                self.consume(TokenKind::CloseParen);
                let body = self.parse_block();
                Statement::While { condition, body }
            }

            TokenKind::Keyword(Keyword::For) => {
                self.advance();
                self.consume(TokenKind::OpenParen);
                let init = Box::new(self.parse_for_clause());
                self.consume(TokenKind::Semicolon);
                let condition = self.parse_expression();
                self.consume(TokenKind::Semicolon);
                let step = Box::new(self.parse_for_clause());
                self.consume(TokenKind::CloseParen);
                let body = self.parse_block();
                Statement::For { init, condition, step, body }
            }

            _ => panic!(
                "Expected 'w' or 'f' after '@', found {:?}",
                self.current().kind
            ),
        }
    }

    fn parse_for_clause(&mut self) -> Statement {
        if self.check(&TokenKind::Hash) {
            return Statement::Variable(self.parse_variable_no_semicolon());
        }

        let expr = self.parse_expression();
        if self.check(&TokenKind::Assign) {
            self.advance();
            let value = self.parse_expression();
            Statement::Assignment { target: expr, value }
        } else {
            Statement::Expression(expr)
        }
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_logical()
    }

    fn parse_logical(&mut self) -> Expression {
        let mut left = self.parse_comparison();

        loop {
            let operator = match &self.current().kind {
                TokenKind::Pipe => BinaryOperator::And,
                TokenKind::PipePipe => BinaryOperator::Or,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison();
            left = Expression::Binary { left: Box::new(left), operator, right: Box::new(right) };
        }

        left
    }

    fn parse_comparison(&mut self) -> Expression {
        let mut left = self.parse_additive();

        loop {
            let operator = match &self.current().kind {
                TokenKind::Equal => BinaryOperator::Equal,
                TokenKind::NotEqual => BinaryOperator::NotEqual,
                TokenKind::Greater => BinaryOperator::Greater,
                TokenKind::Less => BinaryOperator::Less,
                TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenKind::LessEqual => BinaryOperator::LessEqual,
                TokenKind::NotGreater => BinaryOperator::NotGreater,
                TokenKind::NotLess => BinaryOperator::NotLess,
                TokenKind::NotGreaterEqual => BinaryOperator::NotGreaterEqual,
                TokenKind::NotLessEqual => BinaryOperator::NotLessEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive();
            left = Expression::Binary { left: Box::new(left), operator, right: Box::new(right) };
        }

        left
    }

    fn parse_additive(&mut self) -> Expression {
        let mut left = self.parse_multiplicative();

        loop {
            let operator = match &self.current().kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative();
            left = Expression::Binary { left: Box::new(left), operator, right: Box::new(right) };
        }

        left
    }

    fn parse_multiplicative(&mut self) -> Expression {
        let mut left = self.parse_unary();

        loop {
            let operator = match &self.current().kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary();
            left = Expression::Binary { left: Box::new(left), operator, right: Box::new(right) };
        }

        left
    }

    fn parse_unary(&mut self) -> Expression {
        match &self.current().kind {
            TokenKind::Exclamation => {
                self.advance();
                let value = self.parse_unary();
                Expression::Unary { operator: UnaryOperator::Not, value: Box::new(value) }
            }
            TokenKind::Question => {
                self.advance();
                let value = self.parse_unary();
                Expression::Unary { operator: UnaryOperator::Truthy, value: Box::new(value) }
            }
            TokenKind::Minus => {
                self.advance();
                let value = self.parse_unary();
                Expression::Unary { operator: UnaryOperator::Negative, value: Box::new(value) }
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Expression {
        let mut expr = self.parse_primary();

        loop {
            match &self.current().kind {
                TokenKind::Dot => {
                    if matches!(
                        self.tokens.get(self.pos + 1).map(|t| &t.kind),
                        Some(TokenKind::Hash)
                    ) {
                        let is_mutation_keyword = matches!(
                            self.tokens.get(self.pos + 2).map(|t| &t.kind),
                            Some(TokenKind::Identifier(name)) if name == "new" || name == "null"
                        );

                        if is_mutation_keyword {
                            break;
                        }

                        self.advance();
                        self.advance();
                        let target_type = self.parse_type();
                        expr = Expression::Cast { value: Box::new(expr), target_type };
                        continue;
                    }

                    self.advance();

                    let member = match &self.current().kind {
                        TokenKind::Identifier(name) => {
                            let n = name.clone();
                            self.advance();
                            n
                        }
                        TokenKind::String(text) => {
                            let n = text.clone();
                            self.advance();
                            n
                        }
                        _ => panic!(
                            "Expected an identifier after '.', found {:?}",
                            self.current().kind
                        ),
                    };

                    if self.check(&TokenKind::OpenParen) {
                        let arguments = self.parse_arguments();
                        expr = Expression::Call {
                            object: Some(Box::new(expr)),
                            function: member,
                            arguments,
                        };
                    } else {
                        expr = Expression::Member { object: Box::new(expr), member };
                    }
                }

                TokenKind::OpenBracket => {
                    self.advance();
                    let index = self.parse_expression();
                    self.consume(TokenKind::CloseBracket);
                    expr = Expression::ArrayIndex { array: Box::new(expr), index: Box::new(index) };
                }

                TokenKind::OpenParen => {
                    if let Expression::Variable(name) = &expr {
                        let name = name.clone();
                        let arguments = self.parse_arguments();
                        expr = Expression::Call { object: None, function: name, arguments };
                    } else {
                        break;
                    }
                }

                TokenKind::Hash => {
                    self.advance();
                    let target_type = self.parse_type();
                    expr = Expression::Cast { value: Box::new(expr), target_type };
                }

                _ => break,
            }
        }

        expr
    }

    fn parse_arguments(&mut self) -> Vec<Expression> {
        self.consume(TokenKind::OpenParen);
        let mut arguments = Vec::new();

        if !self.check(&TokenKind::CloseParen) {
            arguments.push(self.parse_expression());
            while self.check(&TokenKind::Comma) {
                self.advance();
                arguments.push(self.parse_expression());
            }
        }

        self.consume(TokenKind::CloseParen);
        arguments
    }

    fn parse_primary(&mut self) -> Expression {
        match &self.current().kind {
            TokenKind::Number(n) => {
                let n = *n;
                self.advance();
                Expression::Value(ValueNode::Int(n))
            }

            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                Expression::Value(ValueNode::Float(n))
            }

            TokenKind::Bool(b) => {
                let b = *b;
                self.advance();
                Expression::Value(ValueNode::Bool(b))
            }

            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Expression::Value(ValueNode::String(s))
            }

            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Expression::Variable(name)
            }

            TokenKind::Function(name) => {
                let name = name.clone();
                self.advance();
                Expression::Variable(name)
            }

            TokenKind::OpenParen => {
                self.advance();
                let expr = self.parse_expression();
                self.consume(TokenKind::CloseParen);
                expr
            }

            TokenKind::OpenBracket => {
                self.advance();
                let mut items = Vec::new();

                if !self.check(&TokenKind::CloseBracket) {
                    items.push(self.parse_expression());
                    while self.check(&TokenKind::Comma) {
                        self.advance();
                        if self.check(&TokenKind::CloseBracket) {
                            break;
                        }
                        items.push(self.parse_expression());
                    }
                }

                self.consume(TokenKind::CloseBracket);
                Expression::Array(items)
            }

            TokenKind::OpenBrace => self.parse_json_literal(),

            _ => panic!("Unexpected token in expression: {:?}", self.current().kind),
        }
    }
}
