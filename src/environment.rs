use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::LoxError, interpreter::Value, scanner::Token};

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone, PartialEq)]
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

    pub fn get_at(&self, depth: usize, name: &Token) -> Result<Value, LoxError> {
        if depth == 0 {
            self.values
                .get(&name.lexeme)
                .cloned()
                .ok_or(LoxError::Runtime {
                    line: name.line,
                    message: format!("Undefined variable '{}'.", name.lexeme),
                })
        } else {
            self.enclosing
                .as_ref()
                .expect("Resolved depth is too deep")
                .borrow()
                .get_at(depth - 1, name)
        }
    }

    pub fn assign(&mut self, name: Token, value: Value) -> Result<(), LoxError> {
        if let Some(existing) = self.values.get_mut(&name.lexeme) {
            *existing = value;
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

    pub fn assign_at(&mut self, depth: usize, name: &Token, value: Value) -> Result<(), LoxError> {
        if depth == 0 {
            let val = self.values.get_mut(&name.lexeme).ok_or(LoxError::Runtime {
                line: name.line,
                message: format!("Undefined variable '{}'.", name.lexeme),
            })?;
            *val = value;
            return Ok(());
        }
        self.enclosing
            .as_ref()
            .expect("Resolved depth is too deep")
            .borrow_mut()
            .assign_at(depth - 1, name, value)?;
        Ok(())
    }

    pub fn enclosed(parent: EnvRef) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(parent),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
