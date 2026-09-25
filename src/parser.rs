use crate::{
    ast::Expr,
    error::LoxError,
    scanner::{Literal, Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, current: usize) -> Self {
        Self { tokens, current }
    }

    pub fn parse(&mut self) -> Result<Expr, LoxError> {
        self.expression()
    }

    pub fn expression(&mut self) -> Result<Expr, LoxError> {
        self.equality()
    }

    pub fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr: Expr = self.comparison()?;
        while self.match_type(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous().expect("must be token");
            let right: Expr = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub fn match_type(&mut self, types: &[TokenType]) -> bool {
        for r#type in types {
            if self.check(r#type) {
                self.advance(); // consumes
                return true;
            }
        }
        return false;
    }

    pub fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
            .expect("after advancing, we should be able to see previous")
    }

    pub fn check(&self, r#type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        match self.peek() {
            Some(token) => return &token.token_type == r#type,
            None => return false,
        }
    }

    pub fn peek(&self) -> Option<Token> {
        self.tokens.get(self.current).cloned()
    }

    pub fn previous(&self) -> Option<Token> {
        self.current
            .checked_sub(1)
            .and_then(|i| self.tokens.get(i))
            .cloned()
    }

    pub fn is_at_end(&self) -> bool {
        match self.peek() {
            Some(x) => x.token_type == TokenType::Eof,
            None => false,
        }
    }

    pub fn comparison(&mut self) -> Result<Expr, LoxError> {
        let mut expr: Expr = self.term()?;
        while self.match_type(&[
            TokenType::GreaterEqual,
            TokenType::Greater,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator: Token = self.previous().expect("must be token");
            let right: Expr = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    pub fn term(&mut self) -> Result<Expr, LoxError> {
        let mut expr: Expr = self.factor()?;
        while self.match_type(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous().expect("must be token");
            let right: Expr = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    pub fn factor(&mut self) -> Result<Expr, LoxError> {
        let mut expr: Expr = self.unary()?;
        while self.match_type(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous().expect("must be token");
            let right: Expr = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    pub fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.match_type(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = self.previous().expect("must be token");
            let right: Expr = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    pub fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.match_type(&[TokenType::False]) {
            return Ok(Expr::Literal {
                value: Some(Literal::Bool(false)),
            });
        }

        if self.match_type(&[TokenType::True]) {
            return Ok(Expr::Literal {
                value: Some(Literal::Bool(true)),
            });
        }

        if self.match_type(&[TokenType::Nil]) {
            return Ok(Expr::Literal {
                value: Some(Literal::Nil),
            });
        }

        if self.match_type(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal {
                value: self.previous().expect("must be previous").literal,
            });
        }

        if self.match_type(&[TokenType::LeftParen]) {
            let expr: Expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect '(' after expression")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }
        Err(LoxError::Parse {
            line: self.peek().map(|t| t.line).expect("Should be EOF"),
            message: "Expect expression".to_string(),
        })
    }

    pub fn consume(&mut self, r#type: TokenType, message: &str) -> Result<Token, LoxError> {
        if self.check(&r#type) {
            return Ok(self.advance());
        }
        let line = self.peek().expect("should be EOF").line;
        Err(LoxError::Parse {
            line,
            message: message.to_string(),
        })
    }

    // finding statement boundaries
    pub fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self
                .previous()
                .expect("after advance should be previous")
                .token_type
                == TokenType::Semicolon
            {
                return;
            }
            match self.peek().expect("should work, worst case Eof").token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
