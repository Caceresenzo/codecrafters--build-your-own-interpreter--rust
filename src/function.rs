use crate::{EvaluateInterpreterResult, Interpreter, Statement, Token, Value};

pub trait Callable: std::fmt::Debug {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        token: Token,
    ) -> EvaluateInterpreterResult;
    fn as_str(&self) -> String;
}

#[derive(Debug, PartialEq)]
pub struct LoxFunction {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Statement>,
}

impl LoxFunction {
    pub fn get_name(&self) -> &str {
        &self.name.lexeme
    }
}

impl super::Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        _: Token,
    ) -> EvaluateInterpreterResult {
        let mut environment = interpreter.environment.enclose();

        for (parameter, value) in self.parameters.iter().zip(arguments.into_iter()) {
            environment.define(parameter.lexeme.clone(), value);
        }

        interpreter.execute_block(self.body.clone(), environment)?;
        Ok(Value::Nil)
    }

    fn as_str(&self) -> String {
        format!("<fn {}>", self.name.lexeme)
    }
}

pub mod native {
    use crate::{EvaluateInterpreterResult, Interpreter, InterpreterError, Token, Value};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, PartialEq)]
    pub struct ClockFunction {}

    impl super::Callable for ClockFunction {
        fn arity(&self) -> usize {
            0
        }

        fn call(
            &self,
            _: &mut Interpreter,
            _: Vec<Value>,
            token: Token,
        ) -> EvaluateInterpreterResult {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => Ok(Value::Number(duration.as_secs() as f64)),
                Err(error) => Err(InterpreterError {
                    token: Some(token),
                    message: format!("SystemTime error: {}", error),
                }),
            }
        }

        fn as_str(&self) -> String {
            format!("<native fn {}>", "clock")
        }
    }
}
