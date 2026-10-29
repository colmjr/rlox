use crate::{
    ast::{Expr, Stmt},
    error::LoxError,
    scanner::{Literal, Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize = 0,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, .. }
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
        let expr: Expr = self.or()?;

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

    fn or(&mut self) -> Result<Expr, LoxError> {
        let mut expr: Expr = self.and()?;

        while self.match_type(&[TokenType::Or]) {
            let operator = self.previous().expect("Should be previous.");
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxError> {
        let mut expr: Expr = self.ternary()?;

        while self.match_type(&[TokenType::And]) {
            let operator = self.previous().expect("Should be previous");
            let right = self.ternary()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
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
        self.call()
    }

    fn call(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_type(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LoxError> {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(LoxError::Runtime {
                        line: self.peek().expect("There should be next").line,
                        message: "Can't have more than 255 arguments".to_string(),
                    });
                }
                arguments.push(self.expression()?);
                if !self.match_type(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
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
                | TokenType::Break
                | TokenType::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if self.match_type(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.match_type(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_type(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_type(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }
        if self.match_type(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_type(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.match_type(&[TokenType::Break]) {
            self.consume(TokenType::Semicolon, "Expect ';' after 'break'")?;
            return Ok(Stmt::Break);
        }
        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.)")?;

        let initializer: Option<Stmt> = if self.match_type(&[TokenType::Semicolon]) {
            None
        } else if self.match_type(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition: Option<Expr> = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment: Option<Expr> = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses")?;

        let mut body = self.statement()?;

        if let Some(incr) = increment {
            body = Stmt::Block {
                statements: vec![body, Stmt::Expression { expression: incr }],
            }
        }

        if condition.is_none() {
            condition = Some(Expr::Literal {
                value: Literal::Bool(true),
            })
        }

        body = Stmt::While {
            condition: condition.expect("Invariant not enforced that None condition is true"),
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block {
                statements: vec![init, body],
            }
        }

        Ok(body)
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

    fn if_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'if' condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_type(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' before 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'while'.")?;
        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn declaration(&mut self) -> Result<Stmt, LoxError> {
        if self.match_type(&[TokenType::Fun]) {
            return self.function("function".to_string());
        }
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

    fn function(&mut self, kind: String) -> Result<Stmt, LoxError> {
        // coerce to &str
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;
        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(LoxError::Runtime {
                        line: name.line,
                        message: "Cant have more than 255 params.".to_string(),
                    });
                }
                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
                if !self.match_type(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body", kind),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function {
            name,
            params: parameters,
            body,
        })
    }

    fn return_statement(&mut self) -> Result<Stmt, LoxError> {
        let keyword = self.previous().expect("Should be previous");
        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value")?;

        Ok(Stmt::Return { keyword, value })
    }
}
