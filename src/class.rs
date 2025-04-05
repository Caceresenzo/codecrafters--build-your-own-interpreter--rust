use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{EvaluateInterpreterResult, InterpreterError, LoxFunction, Token, Value};

#[derive(Debug, PartialEq)]
pub struct Class {
    name: String,
    superclass: Option<Rc<RefCell<Class>>>,
    methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
}

impl Class {
    pub fn new(
        name: String,
        superclass: Option<Rc<RefCell<Class>>>,
        methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
    ) -> Self {
        Class {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_function(&self, name: String) -> Option<Rc<RefCell<LoxFunction>>> {
        let method = self.methods.get(&name);
        if let Some(rc) = method {
            return Some(rc.clone());
        }

        if let Some(rc) = self.superclass.clone() {
            return rc.borrow().find_function(name);
        }

        None
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

    pub fn get(&self, name: &Token, self_instance_rc: Rc<RefCell<Instance>>) -> EvaluateInterpreterResult {
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
