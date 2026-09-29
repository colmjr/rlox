use crate::{
    ast::Expr,
    error::LoxError,
    scanner::{Literal, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Number(num) => write!(f, "{num}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn eval(&self, expr: &Expr) -> Result<Value, LoxError> {
        match expr {
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
        }
    }

    pub fn intepret(&self, expr: &Expr) -> Result<(), LoxError> {
        let value = self.eval(expr)?;
        println!("{value}");
        Ok(())
    }
}

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Nil => false,
        _ => true,
    }
}
