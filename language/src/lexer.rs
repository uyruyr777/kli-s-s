use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {

    Number(i64),
    Float(f64),
    String(String),
    Bool(bool),

    Identifier(String),
    Function(String),
    Class(String),

    Keyword(Keyword),

    Type(DataType),

    Colon,
    Comma,
    Dot,
    Semicolon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,

    Assign,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,

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

    Pipe,
    PipePipe,

    Hash,
    At,

    Question,
    Exclamation,

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Import,
    Plugin,
    Start,
    Update,
    While,
    For,
    Return,
    Break,
    Continue,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Float,
    Bool,
    String,
    Ncti,
    Json,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current: Option<char>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {

    pub fn new(source: String) -> Lexer<'static> {
        let source: &'static str = Box::leak(source.into_boxed_str());
        let mut chars = source.chars().peekable();
        let current = chars.next();

        Lexer {
            input: chars,
            current,
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) {
        if let Some(c) = self.current {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.current = self.input.next();
    }

    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(c) = self.current {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_number(&mut self) -> TokenKind {
        let mut text = String::new();

        while let Some(c) = self.current {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if self.current == Some('.') {
            if let Some(next) = self.peek() {
                if next.is_ascii_digit() {
                    text.push('.');
                    self.advance();

                    while let Some(c) = self.current {
                        if c.is_ascii_digit() {
                            text.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    return TokenKind::Float(text.parse().unwrap());
                }
            }
        }

        TokenKind::Number(text.parse().unwrap())
    }

    fn read_string(&mut self) -> TokenKind {
        self.advance();

        let mut text = String::new();

        while let Some(c) = self.current {
            if c == '"' {
                break;
            }
            text.push(c);
            self.advance();
        }

        self.advance();

        TokenKind::String(text)
    }

    fn read_identifier(&mut self) -> String {
        let mut id = String::new();

        while let Some(c) = self.current {
            if c.is_ascii_alphanumeric() || c == '_' {
                id.push(c);
                self.advance();
            } else {
                break;
            }
        }

        id
    }

    fn read_function(&mut self) -> TokenKind {
        self.advance();
        let name = self.read_identifier();
        TokenKind::Function(name)
    }

    fn read_class(&mut self) -> TokenKind {
        self.advance();
        let name = self.read_identifier();
        TokenKind::Class(name)
    }

    fn read_operator(&mut self) -> Option<TokenKind> {
        let ch = self.current?;

        match ch {
            ':' => { self.advance(); Some(TokenKind::Colon) }
            ';' => { self.advance(); Some(TokenKind::Semicolon) }
            ',' => { self.advance(); Some(TokenKind::Comma) }
            '.' => { self.advance(); Some(TokenKind::Dot) }
            '(' => { self.advance(); Some(TokenKind::OpenParen) }
            ')' => { self.advance(); Some(TokenKind::CloseParen) }
            '{' => { self.advance(); Some(TokenKind::OpenBrace) }
            '}' => { self.advance(); Some(TokenKind::CloseBrace) }
            '[' => { self.advance(); Some(TokenKind::OpenBracket) }
            ']' => { self.advance(); Some(TokenKind::CloseBracket) }
            '#' => { self.advance(); Some(TokenKind::Hash) }
            '@' => { self.advance(); Some(TokenKind::At) }
            '+' => { self.advance(); Some(TokenKind::Plus) }
            '-' => { self.advance(); Some(TokenKind::Minus) }
            '*' => { self.advance(); Some(TokenKind::Star) }
            '/' => { self.advance(); Some(TokenKind::Slash) }
            '%' => { self.advance(); Some(TokenKind::Percent) }
            '=' => { self.advance(); Some(TokenKind::Assign) }

            '|' => {
                self.advance();
                if self.current == Some('|') {
                    self.advance();
                    Some(TokenKind::PipePipe)
                } else {
                    Some(TokenKind::Pipe)
                }
            }

            '?' => {
                self.advance();
                match self.current {
                    Some('=') => { self.advance(); Some(TokenKind::Equal) }
                    Some('>') => {
                        self.advance();
                        if self.current == Some('=') {
                            self.advance();
                            Some(TokenKind::GreaterEqual)
                        } else {
                            Some(TokenKind::Greater)
                        }
                    }
                    Some('<') => {
                        self.advance();
                        if self.current == Some('=') {
                            self.advance();
                            Some(TokenKind::LessEqual)
                        } else {
                            Some(TokenKind::Less)
                        }
                    }
                    _ => Some(TokenKind::Question),
                }
            }

            '!' => {
                self.advance();
                match self.current {
                    Some('=') => { self.advance(); Some(TokenKind::NotEqual) }
                    Some('>') => {
                        self.advance();
                        if self.current == Some('=') {
                            self.advance();
                            Some(TokenKind::NotGreaterEqual)
                        } else {
                            Some(TokenKind::NotGreater)
                        }
                    }
                    Some('<') => {
                        self.advance();
                        if self.current == Some('=') {
                            self.advance();
                            Some(TokenKind::NotLessEqual)
                        } else {
                            Some(TokenKind::NotLess)
                        }
                    }
                    _ => Some(TokenKind::Exclamation),
                }
            }

            _ => None,
        }
    }

    fn keyword_or_identifier(&self, text: String) -> TokenKind {
        match text.as_str() {
            "i" => TokenKind::Keyword(Keyword::Import),
            "a" => TokenKind::Keyword(Keyword::Plugin),
            "start" => TokenKind::Keyword(Keyword::Start),
            "update" => TokenKind::Keyword(Keyword::Update),
            "w" => TokenKind::Keyword(Keyword::While),
            "f" => TokenKind::Keyword(Keyword::For),
            "return" => TokenKind::Keyword(Keyword::Return),
            "break" => TokenKind::Keyword(Keyword::Break),
            "continue" => TokenKind::Keyword(Keyword::Continue),
            "exetr" => TokenKind::Keyword(Keyword::Break),
            "exets" => TokenKind::Keyword(Keyword::Exit),
            "int" => TokenKind::Type(DataType::Int),
            "float" => TokenKind::Type(DataType::Float),
            "bool" => TokenKind::Type(DataType::Bool),
            "string" => TokenKind::Type(DataType::String),
            "str" => TokenKind::Type(DataType::String),
            "ncti" => TokenKind::Type(DataType::Ncti),
            "json" => TokenKind::Type(DataType::Json),
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Identifier(text),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            let current = match self.current {
                Some(c) => c,
                None => break,
            };

            let line = self.line;
            let column = self.column;

            if current == '/' && self.peek() == Some('/') {
                self.advance();
                self.advance();
                self.skip_comment();
                continue;
            }

            // Числа
            if current.is_ascii_digit() {
                tokens.push(Token { kind: self.read_number(), line, column });
                continue;
            }

            // Строки
            if current == '"' {
                tokens.push(Token { kind: self.read_string(), line, column });
                continue;
            }

            // Функции $name
            if current == '$' {
                tokens.push(Token { kind: self.read_function(), line, column });
                continue;
            }

            // Классы &Name
            if current == '&' {
                tokens.push(Token { kind: self.read_class(), line, column });
                continue;
            }

            if current.is_ascii_alphabetic() || current == '_' {
                let text = self.read_identifier();
                tokens.push(Token { kind: self.keyword_or_identifier(text), line, column });
                continue;
            }

            if let Some(kind) = self.read_operator() {
                tokens.push(Token { kind, line, column });
                continue;
            }

            panic!("Unknown character '{}' ({}:{})", current, line, column);
        }

        tokens.push(Token {
            kind: TokenKind::EOF,
            line: self.line,
            column: self.column,
        });

        tokens
    }
}
