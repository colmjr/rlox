use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::Stmt,
    class::LoxInstance,
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
    is_init: bool= false,
}

impl LoxFunction {
    pub fn new(declaration: Stmt, closure: EnvRef, is_init: bool) -> Self {
        assert!(matches!(&declaration, Stmt::Function { .. }));
        Self {
            declaration,
            closure,
            is_init,
        }
    }

    pub fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> Self {
        let mut environment = Environment::enclosed(self.closure.clone());
        environment.define("this".to_string(), Value::Instance(instance));
        Self {
            declaration: self.declaration.clone(),
            closure: Rc::new(RefCell::new(environment)),
            is_init: self.is_init,
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

        if self.is_init {
            return Ok(self
                .closure
                .borrow()
                .get_at(0, "this")
                .expect("A bound initializer must have a 'this' binding"));
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
