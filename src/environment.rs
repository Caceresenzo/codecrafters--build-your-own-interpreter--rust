use {
    crate::{EvaluateInterpreterResult, InterpreterError, Literal, Token},
    std::{cell::RefCell, collections::HashMap, rc::Rc},
};

// Thanks https://github.com/Pvlerick/codecrafters-interpreter-rust/blob/master/src/environment.rs

#[derive(Debug, PartialEq, Clone)]
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

    pub fn define(&mut self, name: String, value: Literal) {
        self.inner.borrow_mut().define(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: &Literal) -> Result<(), InterpreterError> {
        self.inner.borrow_mut().assign(name, value)
    }

    pub fn get(&self, name: &Token) -> EvaluateInterpreterResult {
        self.inner.borrow_mut().get(name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Inner {
    enclosing: Option<Rc<RefCell<Inner>>>,
    values: HashMap<String, Literal>,
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

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: &Literal) -> Result<(), InterpreterError> {
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

    pub fn get(&self, name: &Token) -> EvaluateInterpreterResult {
        let lexeme = &name.lexeme;
        if let Some(value) = self.values.get(lexeme) {
            return Ok(value.clone());
        }

        if let Some(parent) = &self.enclosing {
            return parent.borrow().get(name);
        }

        Err(InterpreterError {
            token: Some(name.clone()),
            message: format!("Undefined variable '{lexeme}'."),
        })
    }
}
