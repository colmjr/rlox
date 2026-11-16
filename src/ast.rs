use crate::scanner::{Literal, Token};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub typ: ExprType,
    pub id: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum ExprType {
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
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This {
        keyword: Token,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Class {
        name: Token,
        methods: Vec<Stmt>,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Break {
        keyword: Token,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.typ)
    }
}

impl fmt::Display for ExprType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprType::Assign { name, value } => {
                write!(f, "({} = {})", name.lexeme, value)
            }
            ExprType::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator.lexeme, left, right)
            }
            ExprType::Unary { operator, right } => {
                write!(f, "({} {})", operator.lexeme, right)
            }
            ExprType::Literal { value } => {
                write!(f, "{value}")
            }
            ExprType::Logical {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator.lexeme, left, right)
            }
            ExprType::Grouping { expression } => {
                write!(f, "(group {expression})")
            }
            ExprType::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                write!(f, "({condition} ? {then_branch} : {else_branch})")
            }
            ExprType::Var { name } => {
                write!(f, "var {name}")
            }
            ExprType::Call {
                callee,
                paren: _,
                arguments,
            } => {
                write!(f, "({callee} ({arguments:?}))")
            }
            ExprType::Get { object, name } => {
                write!(f, "get {object} {name}")
            }
            ExprType::Set {
                object,
                name,
                value,
            } => {
                write!(f, "set {object} {name} as {value}")
            }
            ExprType::This { .. } => {
                write!(f, "this")
            }
        }
    }
}

impl Expr {
    pub fn new(typ: ExprType, id: ExprId) -> Self {
        Expr { typ, id }
    }
}
