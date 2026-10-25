use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{Expr, Stmt},
    callable::{Callable, Clock, LoxFunction},
    environment::{EnvRef, Environment},
    error::LoxError,
    scanner::{Literal, TokenType},
};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Callable(Rc<dyn Callable>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    Normal,
    Break,
    Return(Value),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Callable(a), Value::Callable(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Number(num) => write!(f, "{num}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::Callable(_) => write!(f, "Callable(<fn>)"),
        }
    }
}

pub struct Interpreter {
    // current environment
    pub environment: EnvRef,
    pub globals: EnvRef,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        let environment = Rc::clone(&globals);
        globals
            .borrow_mut()
            .define("clock".to_string(), Value::Callable(Rc::new(Clock)));
        Self {
            globals,
            environment,
        }
    }
    pub fn execute(&mut self, stmt: &Stmt) -> Result<Flow, LoxError> {
        match stmt {
            Stmt::Block { statements } => {
                let child = Rc::new(RefCell::new(Environment::enclosed(Rc::clone(
                    &self.environment,
                ))));
                self.execute_block(statements, child)
            }
            Stmt::Expression { expression } => {
                self.eval(expression)?;
                Ok(Flow::Normal)
            }
            Stmt::Print { expression } => {
                let value = self.eval(expression)?;
                println!("{value}");
                Ok(Flow::Normal)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&self.eval(condition)?) {
                    return self.execute(then_branch);
                } else if let Some(branch) = else_branch {
                    return self.execute(branch);
                }
                Ok(Flow::Normal)
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.eval(expr)?,
                    None => Value::Nil,
                };
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);
                Ok(Flow::Normal)
            }
            Stmt::While { condition, body } => {
                while is_truthy(&self.eval(condition)?) {
                    let flow = self.execute(body)?;
                    match flow {
                        Flow::Break => break,
                        f @ Flow::Return(_) => return Ok(f),
                        Flow::Normal => {}
                    }
                }

                Ok(Flow::Normal)
            }
            Stmt::Break => Ok(Flow::Break),
            s @ Stmt::Function { name, .. } => {
                let function = LoxFunction::new(s.clone(), Rc::clone(&self.environment));
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), Value::Callable(Rc::new(function)));
                Ok(Flow::Normal)
            }
            Stmt::Return {
                keyword: _keyword,
                value,
            } => {
                let value = match value {
                    Some(val) => self.eval(val)?,
                    None => Value::Nil,
                };
                Ok(Flow::Return(value))
            }
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, LoxError> {
        match expr {
            Expr::Assign { name, value } => {
                let value = self.eval(value)?;
                self.environment
                    .borrow_mut()
                    .assign(name.clone(), value.clone())?;
                Ok(value)
            }
            Expr::Literal { value } => match value {
                Literal::String(s) => Ok(Value::String(s.clone())),
                Literal::Bool(b) => Ok(Value::Bool(*b)),
                Literal::Number(num) => Ok(Value::Number(*num)),
                Literal::Nil => Ok(Value::Nil),
            },
            Expr::Grouping { expression } => self.eval(expression),
            Expr::Unary { operator, right } => {
                let right = self.eval(right)?;
                match operator.token_type {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operand must be a number".to_string(),
                        }),
                    },
                    TokenType::Bang => Ok(Value::Bool(!is_truthy(&right))),
                    _ => unreachable!("Parser only makes unary nodes with bang and minus"),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.eval(left)?;

                // if tokentype is And, and left is false no need to eval right
                if operator.token_type == TokenType::Or {
                    if is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !is_truthy(&left) {
                        return Ok(left);
                    }
                }

                self.eval(right)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.eval(left)?;
                let right = self.eval(right)?;

                match operator.token_type {
                    TokenType::Minus => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operands must be numbers".to_string(),
                        }),
                    },
                    TokenType::Slash => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operands must be numbers".to_string(),
                        }),
                    },
                    TokenType::Star => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operands must be numbers".to_string(),
                        }),
                    },
                    TokenType::Plus => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operands must be two numbers or strings".to_string(),
                        }),
                    },

                    TokenType::Greater => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operands must be numbers".to_string(),
                        }),
                    },

                    TokenType::GreaterEqual => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operands must be numbers".to_string(),
                        }),
                    },

                    TokenType::Less => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operands must be numbers".to_string(),
                        }),
                    },

                    TokenType::LessEqual => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                        _ => Err(LoxError::Runtime {
                            line: operator.line,
                            message: "Operands must be numbers".to_string(),
                        }),
                    },
                    TokenType::EqualEqual => Ok(Value::Bool(left == right)),
                    TokenType::BangEqual => Ok(Value::Bool(left != right)),

                    _ => unreachable!("Parser only gives binary nodes in normal token types"),
                }
            }
            Expr::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.eval(condition)?;
                if is_truthy(&condition) {
                    self.eval(then_branch)
                } else {
                    self.eval(else_branch)
                }
            }
            Expr::Var { name } => self.environment.borrow().get(name),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.eval(callee)?;
                let mut args: Vec<Value> = Vec::new();
                for arg in arguments {
                    args.push(self.eval(arg)?);
                }

                match callee {
                    Value::Callable(func) => {
                        if args.len() != func.arity() {
                            return Err(LoxError::Runtime {
                                line: paren.line,
                                message: format!(
                                    "Expected {} arguments but got {}.",
                                    func.arity(),
                                    args.len()
                                ),
                            });
                        }
                        Ok(func.call(self, args)?)
                    }
                    _ => Err(LoxError::Runtime {
                        line: paren.line,
                        message: "Can only call classes and functions".to_string(),
                    }),
                }
            }
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), LoxError> {
        for statement in statements {
            self.execute(&statement)?;
        }
        Ok(())
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: EnvRef,
    ) -> Result<Flow, LoxError> {
        let prev = std::mem::replace(&mut self.environment, environment);
        let result = (|| -> Result<Flow, LoxError> {
            for statement in statements {
                let flow = self.execute(statement)?;

                if matches!(flow, Flow::Break | Flow::Return(_)) {
                    return Ok(flow);
                }
            }

            Ok(Flow::Normal)
        })();
        self.environment = prev;
        result
    }
}

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Nil => false,
        _ => true,
    }
}
