use crate::{EvaluateInterpreterResult, Interpreter, Token, Value};

pub trait Callable: std::fmt::Debug {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        token: Token,
    ) -> EvaluateInterpreterResult;
    fn name(&self) -> &str;
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

        fn name(&self) -> &str {
            "clock"
        }
    }
}
