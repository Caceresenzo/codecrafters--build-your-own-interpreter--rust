use crate::{ExecuteInterpreterResult, Interpreter, Token, Value};

#[derive(Debug, PartialEq)]
pub struct Class {
    name: String,
}

impl Class {
    pub fn new(name: String) -> Self {
        Class { name }
    }
}

impl super::Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: &mut Interpreter, _: Vec<Value>, _: &Token) -> ExecuteInterpreterResult {
        todo!()
    }

    fn as_str(&self) -> String {
        return self.name.clone();
    }
}
