use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::Stmt,
    environment::{EnvRef, Environment},
    error::LoxError,
    interpreter::{Flow, Interpreter, Value},
};

pub trait Callable: std::fmt::Debug {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>)
    -> Result<Value, LoxError>;
    fn arity(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    declaration: Stmt,
    closure: EnvRef,
}

impl LoxFunction {
    pub fn new(declaration: Stmt, closure: EnvRef) -> Self {
        assert!(matches!(&declaration, Stmt::Function { .. }));
        Self {
            declaration,
            closure,
        }
    }
}

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Stmt::Function { name, .. } = &self.declaration else {
            unreachable!("constructor ensures this is a function stmt");
        };
        write!(f, "fn {}>", name.lexeme)
    }
}

impl Callable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, LoxError> {
        let environment = Rc::new(RefCell::new(Environment::enclosed(Rc::clone(
            &self.closure,
        ))));
        let Stmt::Function { params, body, .. } = &self.declaration else {
            unreachable!("constructor ensures this is a function stmt");
        };

        for i in 0..params.len() {
            environment
                .borrow_mut()
                .define(params[i].clone().lexeme, arguments[i].clone());
        }

        match interpreter.execute_block(body, environment)? {
            Flow::Normal => Ok(Value::Nil),
            Flow::Break => unreachable!("break must not break a function body"),
            Flow::Return(value) => Ok(value),
        }
    }

    fn arity(&self) -> usize {
        let Stmt::Function { params, .. } = &self.declaration else {
            unreachable!("constructor ensures this is a function stmt");
        };
        params.len()
    }
}

#[derive(Debug)]
pub struct Clock;

impl std::fmt::Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Callable for Clock {
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Value>,
    ) -> Result<Value, LoxError> {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Should be after unix epoch");
        Ok(Value::Number(elapsed.as_secs_f64()))
    }

    fn arity(&self) -> usize {
        0
    }
}
