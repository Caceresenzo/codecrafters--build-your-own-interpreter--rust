use crate::{Expression, Literal, Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Interpreter {}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct InterpreterError {
    pub token: Option<Token>,
    pub message: String,
}

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
                    TokenType::Minus => Ok(Literal::Number(
                        -self.check_number_operand(&operator, &right_child)?,
                    )),
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
                    TokenType::Plus => {
                        if let (Literal::Number(a), Literal::Number(b)) =
                            (&left_child, &right_child)
                        {
                            return Ok(Literal::Number(*a + *b));
                        }

                        if let (Literal::String(a), Literal::String(b)) =
                            (&left_child, &right_child)
                        {
                            let mut output = a.to_owned();
                            output.push_str(b);

                            return Ok(Literal::String(output));
                        }

                        Err(InterpreterError {
                            token: None,
                            message: "expected number or string".into(),
                        })
                    }
                    TokenType::Greater => Ok(Literal::Boolean(
                        self.number(left_child)? > self.number(right_child)?,
                    )),
                    TokenType::GreaterEqual => Ok(Literal::Boolean(
                        self.number(left_child)? >= self.number(right_child)?,
                    )),
                    TokenType::Less => Ok(Literal::Boolean(
                        self.number(left_child)? < self.number(right_child)?,
                    )),
                    TokenType::LessEqual => Ok(Literal::Boolean(
                        self.number(left_child)? <= self.number(right_child)?,
                    )),
                    TokenType::BangEqual => Ok(Literal::Boolean(left_child != right_child)),
                    TokenType::EqualEqual => Ok(Literal::Boolean(left_child == right_child)),
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
            _ => Err(InterpreterError {
                token: None,
                message: "expected number".into(),
            }),
        }
    }

    pub fn check_number_operand(
        &self,
        operator: &Token,
        operand: &Literal,
    ) -> Result<f64, InterpreterError> {
        match operand {
            Literal::Number(x) => Ok(*x),
            _ => Err(InterpreterError {
                token: Some(operator.clone()),
                message: "Operand must be a number.".into(),
            }),
        }
    }
}
