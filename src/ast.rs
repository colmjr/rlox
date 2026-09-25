use crate::scanner::{Literal, Token};
use std::fmt;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Literal {
        value: Option<Literal>,
    },
    Grouping {
        expression: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator.lexeme, left, right)
            }
            Expr::Unary { operator, right } => {
                write!(f, "({} {})", operator.lexeme, right)
            }
            Expr::Literal { value } => {
                if let Some(val) = value {
                    write!(f, "{val}")
                } else {
                    write!(f, "nil")
                }
            }
            Expr::Grouping { expression } => {
                write!(f, "(group {expression})")
            }
        }
    }
}
