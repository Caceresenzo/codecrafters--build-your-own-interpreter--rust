use {
    crate::{EvaluateInterpreterResult, InterpreterError, Token, Value},
    std::{cell::RefCell, collections::HashMap, rc::Rc},
};

// Thanks https://github.com/Pvlerick/codecrafters-interpreter-rust/blob/master/src/environment.rs

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    inner: Rc<RefCell<Inner>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::new())),
        }
    }

    pub fn enclose(&self) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::enclose(self.inner.clone()))),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.inner.borrow_mut().define(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: &Value) -> Result<(), InterpreterError> {
        self.inner.borrow_mut().assign(name, value)
    }

    pub fn assign_at(&mut self, distance: u32, name: &Token, value: &Value) -> Result<(), InterpreterError> {
        Ok(self.ancestor(distance).borrow_mut().assign_no_parent(name, value))
    }

    pub fn get(&self, name: &Token) -> EvaluateInterpreterResult {
        self.inner.borrow_mut().get(name)
    }

    pub fn get_at(&self, distance: u32, name: &String) -> EvaluateInterpreterResult {
        self.ancestor(distance).borrow_mut().get_no_parent(name)
    }

    fn ancestor(&self, distance: u32) -> Rc<RefCell<Inner>> {
        let mut environment = Rc::clone(&self.inner);

        for _ in 0..distance {
            let next_environment = {
                let borrowed_env = environment.borrow();
                Rc::clone(borrowed_env.enclosing.as_ref().unwrap())
            };

            environment = next_environment;
        }

        return environment;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Inner {
    enclosing: Option<Rc<RefCell<Inner>>>,
    values: HashMap<String, Value>,
}

impl Inner {
    pub fn new() -> Self {
        Inner {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    fn enclose(inner: Rc<RefCell<Inner>>) -> Self {
        Self {
            enclosing: Some(inner.clone()),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: &Value) -> Result<(), InterpreterError> {
        let lexeme = &name.lexeme;
        if self.values.contains_key(lexeme) {
            self.values.insert(lexeme.clone(), value.clone());
            return Ok(());
        }

        if let Some(parent) = &mut self.enclosing {
            return parent.borrow_mut().assign(name, value);
        }

        Err(InterpreterError {
            token: Some(name.clone()),
            message: format!("Undefined variable '{lexeme}'."),
        })
    }

    pub fn assign_no_parent(&mut self, name: &Token, value: &Value) {
        self.values.insert(name.lexeme.clone(), value.clone());
    }

    pub fn get(&self, name: &Token) -> EvaluateInterpreterResult {
        let lexeme = &name.lexeme;
        if let Some(value) = self.values.get(lexeme) {
            return Ok(value.clone());
        }

        if let Some(parent) = &self.enclosing {
            return parent.borrow().get(name);
        }

        dbg!(&self.values);

        Err(InterpreterError {
            token: Some(name.clone()),
            message: format!("Undefined variable '{lexeme}'."),
        })
    }

    pub fn get_no_parent(&self, name: &String) -> EvaluateInterpreterResult {
        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }

        Ok(Value::Nil)
    }
}
