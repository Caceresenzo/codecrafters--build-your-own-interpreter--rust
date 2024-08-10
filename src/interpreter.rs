use crate::{Expression, Literal, TokenType};

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
            Expression::Unary { operator, right } => {
                let right_child = self.evaluate(*right)?;

                match operator.token_type {
                    TokenType::Bang => Ok(Literal::Boolean(!self.is_truthy(right_child))),
                    TokenType::Minus => match right_child {
                        Literal::Number(x) => Ok(Literal::Number(-x)),
                        _ => Err(InterpreterError("expected number".into())),
                    },
                    _ => panic!("unreachable"),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_child = self.evaluate(*left)?;
                let right_child = self.evaluate(*right)?;

                match operator.token_type {
                    TokenType::Slash => Ok(Literal::Number(
                        self.number(left_child)? / self.number(right_child)?,
                    )),
                    TokenType::Star => Ok(Literal::Number(
                        self.number(left_child)? * self.number(right_child)?,
                    )),
                    TokenType::Minus => Ok(Literal::Number(
                        self.number(left_child)? - self.number(right_child)?,
                    )),
                    TokenType::Plus => Ok(Literal::Number(
                        self.number(left_child)? + self.number(right_child)?,
                    )),
                    _ => panic!("unreachable"),
                }
            }
        }
    }

    pub fn is_truthy(&self, literal: Literal) -> bool {
        match literal {
            Literal::Nil => false,
            Literal::Boolean(value) => value,
            _ => true,
        }
    }

    pub fn number(&self, literal: Literal) -> Result<f64, InterpreterError> {
        match literal {
            Literal::Number(value) => Ok(value),
            _ => Err(InterpreterError("expected number".into())),
        }
    }
}
