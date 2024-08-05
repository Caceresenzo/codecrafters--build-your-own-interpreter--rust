use std::fmt;

use crate::{Literal, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Grouping(Box<Expression>),
    Unary(Token, Box<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(value) => write!(f, "{value}"),
            Expression::Grouping(expression) => write!(f, "(group {expression})"),
            Expression::Unary(operator, expression) => write!(f, "({} {expression})", operator.lexeme),
        }
    }
}
