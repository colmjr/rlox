use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    callable::{Callable, LoxFunction},
    error::LoxError,
    interpreter::{Interpreter, Value},
    scanner::Token,
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, LoxFunction>,
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    pub class: Rc<LoxClass>,
    pub fields: HashMap<String, Value>,
}

impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Callable for Rc<LoxClass> {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, LoxError> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(Rc::clone(self))));
        let initializer = self.find_method("init".to_string());
        if let Some(init) = initializer {
            init.bind(Rc::clone(&instance))
                .call(interpreter, arguments)?;
        }
        Ok(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method("init".to_string());
        if let Some(init) = initializer {
            return init.arity();
        }
        0
    }
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: String) -> Option<LoxFunction> {
        self.methods.get(&name).cloned()
    }
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(instance: Rc<RefCell<Self>>, name: &Token) -> Result<Value, LoxError> {
        let method = {
            let borrowed = instance.borrow();

            if let Some(value) = borrowed.fields.get(&name.lexeme) {
                return Ok(value.clone());
            }

            borrowed.class.find_method(name.lexeme.clone())
        };

        if let Some(method) = method {
            let bound = method.bind(Rc::clone(&instance));
            return Ok(Value::Callable(Rc::new(bound)));
        }

        Err(LoxError::Runtime {
            line: name.line,
            message: format!("Undefined property \"{}\" ", name.lexeme),
        })
    }

    pub fn set(&mut self, name: Token, value: Value) {
        self.fields.insert(name.lexeme, value);
    }
}
