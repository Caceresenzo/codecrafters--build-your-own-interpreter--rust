use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{EvaluateInterpreterResult, InterpreterError, LoxFunction, Token, Value};

#[derive(Debug, PartialEq)]
pub struct Class {
    name: String,
    methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Rc<RefCell<LoxFunction>>>) -> Self {
        Class { name, methods }
    }

    pub fn find_function(&self, name: String) -> Option<&Rc<RefCell<LoxFunction>>> {
        return self.methods.get(&name);
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

    pub fn get(&self, name: &Token, self_instance_rc: &Value) -> EvaluateInterpreterResult {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(function) = self.class.borrow().find_function(name.lexeme.clone()) {
            return Ok(Value::Function(Rc::new(RefCell::new(
                function.borrow().bind(self_instance_rc.clone()),
            ))));
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
