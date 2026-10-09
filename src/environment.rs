use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::LoxError, interpreter::Value, scanner::Token};

pub type EnvRef = Rc<RefCell<Environment>>;

pub struct Environment {
    pub values: HashMap<String, Value>,
    pub enclosing: Option<EnvRef>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        // throw away previous value
        let _ = self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, LoxError> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(enc) => enc.borrow().get(name),
                None => Err(LoxError::Runtime {
                    line: name.line,
                    message: format!("Undefined variable '{}'.", name.lexeme),
                }),
            },
        }
    }

    pub fn assign(&mut self, name: Token, value: Value) -> Result<(), LoxError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(());
        }

        match &self.enclosing {
            Some(enc) => enc.borrow_mut().assign(name, value),
            None => Err(LoxError::Runtime {
                line: name.line,
                message: format!("Undefined variable '{}'.", name.lexeme),
            }),
        }
    }

    pub fn enclosed(parent: EnvRef) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(parent),
        }
    }
}
