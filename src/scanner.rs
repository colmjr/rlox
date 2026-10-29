use crate::error::LoxError;
use std::collections::HashMap;
use std::fmt;

pub struct Scanner<'a> {
    source: &'a [char],
    tokens: Vec<Token> = Vec::new(),
    start: usize = 0,
    current: usize = 0,
    line: usize = 0,
    keywords: HashMap<&'a str, TokenType>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Colon,
    Semicolon,
    Slash,
    Question,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Break,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::String(string) => write!(f, "\"{}\"", string),

            Literal::Number(num) => write!(f, "{}", num),

            Literal::Bool(bool) => write!(f, "{}", bool),

            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a [char]) -> Scanner<'a> {
        let map = HashMap::from([
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
            ("break", TokenType::Break),
        ]);

        Self {
            source,
            keywords: map,
            ..
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // The reason for the following function signature is that we would like the currently
    // parsed tokens, and this may be useful to see where it errored instead of a Result<>
    pub fn scan_tokens(&mut self) -> (Vec<Token>, Vec<LoxError>) {
        let mut errors: Vec<LoxError> = Vec::new();
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token(&mut errors);
        }
        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), None, self.line));
        (std::mem::take(&mut self.tokens), errors)
    }

    pub fn scan_token(&mut self, errors: &mut Vec<LoxError>) {
        let c = self.source[self.current];
        self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '?' => self.add_token(TokenType::Question, None),
            ':' => self.add_token(TokenType::Colon, None),
            '!' => {
                let kind = if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(kind, None);
            }
            '=' => {
                let kind = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(kind, None);
            }
            '<' => {
                let kind = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(kind, None);
            }
            '>' => {
                let kind = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(kind, None);
            }
            '/' => {
                // TODO: Add multiline comment support
                if self.match_next('/') {
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            '"' => self.string(errors),
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            c if c.is_ascii_digit() => self.number(errors),
            c if is_alpha(c) => self.identifier(errors),
            _ => errors.push(LoxError::Scan {
                line: self.line,
                message: "Unexpected character".to_string(),
            }),
        }
    }

    pub fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        self.tokens
            .push(Token::new(token_type, lexeme, literal, self.line));
    }

    pub fn advance(&mut self) {
        self.current += 1;
    }

    pub fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    pub fn peek(&self) -> Option<char> {
        // instead of indexing and possible panic, we use get to return Option
        self.source.get(self.current).copied()
    }

    pub fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            return None;
        }
        self.source.get(self.current + 1).copied()
    }

    pub fn string(&mut self, errors: &mut Vec<LoxError>) {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            };
            self.advance();
        }

        if self.is_at_end() {
            errors.push(LoxError::Scan {
                line: self.line,
                message: "Unterminated String".to_string(),
            });
            return;
        }

        // closing "
        self.advance();

        let value: String = self.source[(self.start + 1)..(self.current - 1)]
            .iter()
            .collect();
        self.add_token(TokenType::String, Some(Literal::String(value)));
    }

    pub fn number(&mut self, errors: &mut Vec<LoxError>) {
        while let Some(val) = self.peek()
            && val.is_ascii_digit()
        {
            self.advance();
        }

        if let Some('.') = self.peek() {
            if let Some(val) = self.peek_next()
                && val.is_ascii_digit()
            {
                self.advance();

                while let Some(val) = self.peek()
                    && val.is_ascii_digit()
                {
                    self.advance();
                }
            } else {
                errors.push(LoxError::Scan {
                    line: self.line,
                    message: "no fractional part after '.'".to_string(),
                });
            }
        }

        // we expect here because the function calling is under the condition that c.is_ascii_digit()
        let value: f64 = self.source[self.start..self.current]
            .iter()
            .collect::<String>()
            .parse()
            .expect("should be ascii digit");
        self.add_token(TokenType::Number, Some(Literal::Number(value)));
    }

    // TODO: is there a case in which fn identifier fails and we need access to errors?
    pub fn identifier(&mut self, _errors: &mut Vec<LoxError>) {
        while let Some(val) = self.peek()
            && is_alphanumeric(val)
        {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        match self.keywords.get(text.as_str()) {
            Some(&t) => {
                self.add_token(t, None);
            }
            None => {
                self.add_token(TokenType::Identifier, None);
            }
        };
    }
}

pub fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

pub fn is_alphanumeric(c: char) -> bool {
    c.is_ascii_digit() || is_alpha(c)
}
