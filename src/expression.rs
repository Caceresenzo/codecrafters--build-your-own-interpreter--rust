use std::fmt;

use crate::{Literal, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Grouping(Box<Expression>),
    Unary {
        operator: Token,
        right: Box<Expression>
    },
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>
    },
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(value) => write!(f, "{value}"),
            Expression::Grouping(expression) => write!(f, "(group {expression})"),
            Expression::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme),
            Expression::Binary { left, operator, right } => write!(f, "({} {left} {right})", operator.lexeme),
        }
    }
}
