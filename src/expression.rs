use std::fmt;

use crate::Literal;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Grouping(Box<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(value) => write!(f, "{value}"),
            Expression::Grouping(expression) => write!(f, "(group {expression})"),
        }
    }
}
