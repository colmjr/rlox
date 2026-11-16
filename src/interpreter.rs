use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{Expr, ExprId, ExprType, Stmt},
    callable::{Callable, Clock, LoxFunction},
    class::{LoxClass, LoxInstance},
    environment::{EnvRef, Environment},
    error::LoxError,
    scanner::{Literal, Token, TokenType},
};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Callable(Rc<dyn Callable>),
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<LoxInstance>>),
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
            (Value::Class(a), Value::Class(b)) => Rc::ptr_eq(a, b),
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(a, b),
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
            Value::Class(c) => write!(f, "Class {}", c.name),
            Value::Instance(instance) => write!(f, "Instance of {}", instance.borrow().class),
        }
    }
}

pub struct Interpreter {
    // current environment
    pub environment: EnvRef,
    pub globals: EnvRef,
    pub locals: HashMap<ExprId, usize>,
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
        let locals = HashMap::new();
        Self {
            globals,
            environment,
            locals,
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
            Stmt::Break { .. } => Ok(Flow::Break),
            s @ Stmt::Function { name, .. } => {
                let function = LoxFunction::new(s.clone(), Rc::clone(&self.environment), false);
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
            Stmt::Class { name, methods } => {
                let mut env = self.environment.borrow_mut();
                env.define(name.lexeme.clone(), Value::Nil);
                let mut method_map: HashMap<String, LoxFunction> = HashMap::new();
                for method in methods {
                    let Stmt::Function {
                        name: method_name, ..
                    } = method
                    else {
                        unreachable!("class bodies contain only methods");
                    };

                    let function = LoxFunction::new(
                        method.clone(),
                        Rc::clone(&self.environment),
                        method_name.lexeme == "init",
                    );
                    method_map.insert(method_name.lexeme.clone(), function);
                }

                let class =
                    Value::Class(Rc::new(LoxClass::new(name.lexeme.to_string(), method_map)));
                env.assign(name.clone(), class)?;

                Ok(Flow::Normal)
            }
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, LoxError> {
        match &expr.typ {
            ExprType::Assign { name, value } => {
                let value = self.eval(value)?;

                if let Some(distance) = self.locals.get(&expr.id) {
                    self.environment
                        .borrow_mut()
                        .assign_at(*distance, name, value.clone())?;
                } else {
                    self.globals
                        .borrow_mut()
                        .assign(name.clone(), value.clone())?;
                }

                Ok(value)
            }
            ExprType::Literal { value } => match value {
                Literal::String(s) => Ok(Value::String(s.clone())),
                Literal::Bool(b) => Ok(Value::Bool(*b)),
                Literal::Number(num) => Ok(Value::Number(*num)),
                Literal::Nil => Ok(Value::Nil),
            },
            ExprType::Grouping { expression } => self.eval(expression),
            ExprType::Unary { operator, right } => {
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
            ExprType::Logical {
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
            ExprType::Binary {
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
            ExprType::Ternary {
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
            ExprType::Var { name } => self.lookup_var(name, expr),
            ExprType::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.eval(callee)?;
                let mut args: Vec<Value> = Vec::new();
                for arg in arguments {
                    args.push(self.eval(arg)?);
                }

                let callable: &dyn Callable = match &callee {
                    Value::Callable(func) => func.as_ref(),
                    Value::Class(class) => class,

                    _ => {
                        return Err(LoxError::Runtime {
                            line: paren.line,
                            message: "Can only call classes and functions".to_string(),
                        });
                    }
                };

                if args.len() != callable.arity() {
                    return Err(LoxError::Runtime {
                        line: paren.line,
                        message: format!(
                            "Expected {} arguments but got {}.",
                            callable.arity(),
                            args.len()
                        ),
                    });
                }
                callable.call(self, args)
            }
            ExprType::Get { object, name } => {
                let o = self.eval(object)?;
                if let Value::Instance(obj) = o {
                    return LoxInstance::get(obj, name);
                }

                Err(LoxError::Runtime {
                    line: name.line,
                    message: "Only instances have properties".to_string(),
                })
            }
            ExprType::Set {
                object,
                name,
                value,
            } => {
                let object = self.eval(object)?;

                let Value::Instance(instance) = object else {
                    return Err(LoxError::Runtime {
                        line: name.line,
                        message: "Only instances have field".to_string(),
                    });
                };

                let value = self.eval(value)?;
                instance.borrow_mut().set(name.clone(), value.clone());
                Ok(value)
            }
            ExprType::This { keyword } => self.lookup_var(keyword, expr),
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

    pub fn resolve(&mut self, id: ExprId, depth: usize) {
        self.locals.insert(id, depth);
    }

    fn lookup_var(&self, name: &Token, expr: &Expr) -> Result<Value, LoxError> {
        if let Some(&depth) = self.locals.get(&expr.id) {
            self.environment
                .borrow()
                .get_at(depth, &name.lexeme)
                .ok_or_else(|| LoxError::Runtime {
                    line: name.line,
                    message: format!("Undefined variable '{}'.", name.lexeme),
                })
        } else {
            self.globals.borrow().get(name)
        }
    }
}

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Nil => false,
        _ => true,
    }
}
