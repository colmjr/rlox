use crate::{
    ast::{Expr, Stmt},
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<LoxError>> {
        let mut statements: Vec<Stmt> = Vec::new();
        let mut errors: Vec<LoxError> = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    errors.push(err);
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LoxError> {
        let expr: Expr = self.ternary()?;
        if self.match_type(&[TokenType::Equal]) {
            let equals = self.previous().expect("Should be previous.");
            let value = self.assignment()?;
            match expr {
                Expr::Var { name } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    });
                }
                _ => {
                    return Err(LoxError::Parse {
                        line: equals.line,
                        message: "Invalid assignment target".to_string(),
                    });
                }
            }
        }
        Ok(expr)
    }
    fn ternary(&mut self) -> Result<Expr, LoxError> {
        let mut expr: Expr = self.equality()?;
        while self.match_type(&[TokenType::Question]) {
            let then_branch: Expr = self.expression()?;
            self.consume(TokenType::Colon, "expect ':' in ternary")?;
            let else_branch: Expr = self.ternary()?;
            expr = Expr::Ternary {
                condition: Box::new(expr),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
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

    fn comparison(&mut self) -> Result<Expr, LoxError> {
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

    fn term(&mut self) -> Result<Expr, LoxError> {
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

    fn factor(&mut self) -> Result<Expr, LoxError> {
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

    fn unary(&mut self) -> Result<Expr, LoxError> {
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

    fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.match_type(&[TokenType::False]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        }

        if self.match_type(&[TokenType::True]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }

        if self.match_type(&[TokenType::Nil]) {
            return Ok(Expr::Literal {
                value: Literal::Nil,
            });
        }

        if self.match_type(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal {
                value: self
                    .previous()
                    .expect("must be previous")
                    .literal
                    .expect("number, string tokens always carry a literal"),
            });
        }

        if self.match_type(&[TokenType::LeftParen]) {
            let expr: Expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect '(' after expression")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        if self.match_type(&[TokenType::Identifier]) {
            return Ok(Expr::Var {
                name: self.previous().expect("Must be previous"),
            });
        }

        Err(LoxError::Parse {
            line: self.peek().map(|t| t.line).expect("Should be EOF"),
            message: "Expect expression".to_string(),
        })
    }

    fn match_type(&mut self, types: &[TokenType]) -> bool {
        for r#type in types {
            if self.check(r#type) {
                self.advance(); // consumes
                return true;
            }
        }
        false
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
            .expect("after advancing, we should be able to see previous")
    }

    fn check(&self, r#type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        match self.peek() {
            Some(token) => &token.token_type == r#type,
            None => false,
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.current).cloned()
    }

    fn previous(&self) -> Option<Token> {
        self.current
            .checked_sub(1)
            .and_then(|i| self.tokens.get(i))
            .cloned()
    }

    fn is_at_end(&self) -> bool {
        match self.peek() {
            Some(x) => x.token_type == TokenType::Eof,
            None => false,
        }
    }

    fn consume(&mut self, r#type: TokenType, message: &str) -> Result<Token, LoxError> {
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
    fn synchronize(&mut self) {
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

    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if self.match_type(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_type(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxError> {
        let value: Expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
        let expression: Expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Expression { expression })
    }

    fn declaration(&mut self) -> Result<Stmt, LoxError> {
        if self.match_type(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?;

        let initializer: Option<Expr> = if self.match_type(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expected ';'")?;
        Ok(Stmt::Var { name, initializer })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, LoxError> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '{' after block.")?;
        Ok(statements)
    }
}
