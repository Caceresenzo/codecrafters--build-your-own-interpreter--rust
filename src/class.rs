use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{EvaluateInterpreterResult, InterpreterError, Token, Value};

#[derive(Debug, PartialEq)]
pub struct Class {
    name: String,
}

impl Class {
    pub fn new(name: String) -> Self {
        Class { name }
    }

    pub fn as_str(&self) -> String {
        return self.name.clone();
    }
}

#[derive(Debug, PartialEq)]
pub struct Instance {
    class: Rc<RefCell<Class>>,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Rc<RefCell<Class>>) -> Self {
        Instance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> EvaluateInterpreterResult {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone());
        }

        Err(InterpreterError {
            token: Some(name.clone()),
            message: format!("Undefined property '{}'.", name.lexeme),
        })
    }

    pub fn set(&mut self, name: &Token, value: Value) -> EvaluateInterpreterResult {
        self.fields.insert(name.lexeme.clone(), value);

        Ok(Value::Nil)
    }

    pub fn as_str(&self) -> String {
        return format!("{} instance", self.class.borrow().name);
    }
}
