use crate::scanner::{Literal, Token};
use std::fmt;

pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
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
        value: Literal,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Var {
        name: Token,
    },
}

pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Assign { name, value } => {
                write!(f, "({} = {})", name.lexeme, value)
            }
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
                write!(f, "{value}")
            }
            Expr::Grouping { expression } => {
                write!(f, "(group {expression})")
            }
            Expr::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                write!(f, "({condition} ? {then_branch} : {else_branch})")
            }
            Expr::Var { name } => {
                write!(f, "var {name}")
            }
        }
    }
}
