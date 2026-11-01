use std::collections::HashMap;

use crate::{
    ast::{Expr, ExprType, Stmt},
    error::LoxError,
    interpreter::Interpreter,
    scanner::Token,
};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    // bool: whether finished resolving
    scopes: Vec<HashMap<String, bool>>,
    current_func: FunctionType = FunctionType::None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        let scopes: Vec<HashMap<String, bool>> = Vec::new();
        Self {
            interpreter,
            scopes,
            ..
        }
    }

    pub fn resolve_stmts(&mut self, statements: &[Stmt]) -> Result<(), LoxError> {
        for stmt in statements {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), LoxError> {
        match stmt {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_stmts(statements)?;
                self.end_scope();
            }
            Stmt::Expression { expression } => {
                self.resolve_expr(expression)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch)?;
                }
            }
            Stmt::Print { expression } => {
                self.resolve_expr(expression)?;
            }
            Stmt::Var { name, initializer } => {
                self.declare(name)?;
                if let Some(init) = initializer {
                    self.resolve_expr(init)?;
                }
                self.define(name);
            }
            Stmt::Return { keyword, value } => {
                if self.current_func == FunctionType::None {
                    return Err(LoxError::Resolve {
                        line: keyword.line,
                        message: "Can't return from top level code.".to_string(),
                    });
                }
                if let Some(val) = value {
                    self.resolve_expr(val)?;
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
            }
            s @ Stmt::Function {
                name,
                params: _,
                body: _,
            } => {
                self.declare(name)?;
                self.define(name);

                self.resolve_function(s, FunctionType::Function)?;
            }
            Stmt::Break => todo!(),
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("Did not pop scope in resolver");
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), LoxError> {
        match &expr.typ {
            ExprType::Assign { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(value, name)?;
            }
            ExprType::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            ExprType::Unary { operator: _, right } => {
                self.resolve_expr(right)?;
            }
            ExprType::Literal { value: _ } => {}
            ExprType::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            ExprType::Grouping { expression } => {
                self.resolve_expr(expression)?;
            }
            ExprType::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expr(callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }
            }
            ExprType::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_expr(then_branch)?;
                self.resolve_expr(else_branch)?;
            }
            ExprType::Var { name } => {
                if !self.scopes.is_empty()
                    && self
                        .scopes
                        .last_mut()
                        .expect("Scopes is empty")
                        .get(&name.lexeme)
                        == Some(&false)
                {
                    return Err(LoxError::Resolve {
                        line: name.line,
                        message: "Can't read local variable in its own initializer.".to_string(),
                    });
                }

                self.resolve_local(expr, name)?;
            }
        }

        Ok(())
    }

    fn declare(&mut self, name: &Token) -> Result<(), LoxError> {
        if self.scopes.is_empty() {
            return Ok(());
        }
        let scope: &mut HashMap<String, bool> = self.scopes.last_mut().expect("Scope is empty");
        if scope.contains_key(&name.lexeme) {
            return Err(LoxError::Resolve {
                line: name.line,
                message: "Already a variable with this name in this scope.".to_string(),
            });
        }
        scope.insert(name.clone().lexeme, false);
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope: &mut HashMap<String, bool> = self.scopes.last_mut().expect("Scope is empty");
        scope.insert(name.clone().lexeme, true);
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) -> Result<(), LoxError> {
        // innermost
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                // depth = exprid
                self.interpreter.resolve(expr.id, depth);

                return Ok(());
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, stmt: &Stmt, func_type: FunctionType) -> Result<(), LoxError> {
        let enclosing_func = self.current_func;
        self.current_func = func_type;
        self.begin_scope();
        let Stmt::Function {
            name: _,
            params,
            body,
        } = stmt
        else {
            unreachable!("only called on function")
        };
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_stmts(body)?;
        self.end_scope();
        self.current_func = enclosing_func;
        Ok(())
    }
}
