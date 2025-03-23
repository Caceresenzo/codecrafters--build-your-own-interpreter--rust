use crate::{Environment, ExecuteInterpreterResult, FunctionData, Interpreter, Statement, Token, Value};

pub trait Callable: std::fmt::Debug {
    fn arity(&self) -> usize;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        token: &Token,
    ) -> ExecuteInterpreterResult;

    fn as_str(&self) -> String;
}

#[derive(Debug, PartialEq)]
pub struct LoxFunction {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Statement>,
    pub is_initializer: bool,
    pub closure: Environment,
}

impl LoxFunction {
    pub fn new(data: &FunctionData, is_initializer: bool, closure: Environment) -> Self {
        LoxFunction {
            name: data.name.clone(),
            parameters: data.parameters.clone(),
            body: data.body.clone(),
            is_initializer,
            closure,
        }
    }

    pub fn bind(&self, instance_value: Value) -> LoxFunction {
        let mut environment = self.closure.enclose();
        environment.define("this".into(), instance_value);

        LoxFunction {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            is_initializer: self.is_initializer,
            closure: environment,
        }
    }

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
        _: &Token,
    ) -> ExecuteInterpreterResult {
        let mut environment = self.closure.enclose();

        for (parameter, value) in self.parameters.iter().zip(arguments.into_iter()) {
            environment.define(parameter.lexeme.clone(), value);
        }

        let returned = interpreter.execute_block(self.body.as_ref(), environment)?;
        
        if self.is_initializer {
            return Ok(Some(self.closure.get_at(0, "this".into())?))
        }
        
        Ok(returned)
    }

    fn as_str(&self) -> String {
        format!("<fn {}>", self.name.lexeme)
    }
}

pub mod native {
    use crate::{ExecuteInterpreterResult, Interpreter, InterpreterError, Token, Value};
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
            token: &Token,
        ) -> ExecuteInterpreterResult {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => Ok(Some(Value::Number(duration.as_secs() as f64))),
                Err(error) => Err(InterpreterError {
                    token: Some(token.clone()),
                    message: format!("SystemTime error: {}", error),
                }),
            }
        }

        fn as_str(&self) -> String {
            format!("<native fn {}>", "clock")
        }
    }
}
