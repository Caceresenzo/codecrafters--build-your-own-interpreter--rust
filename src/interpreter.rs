use crate::{Expression, Literal};

#[derive(Debug, PartialEq, Clone)]
pub struct Interpreter {}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct InterpreterError(String);

type InterpreterResult = Result<Literal, InterpreterError>;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn evaluate(&self, expression: Expression) -> InterpreterResult {
        match expression {
            Expression::Literal(literal) => Ok(literal),
            Expression::Grouping(child) => self.evaluate(*child),
            _ => Err(InterpreterError("unsupported".into())),
        }
    }
}
