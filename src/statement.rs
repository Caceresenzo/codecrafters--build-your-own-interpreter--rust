use std::fmt;

use crate::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Expression(expression) => write!(f, "{expression}"),
            Statement::Print(expression) => write!(f, "{expression}"),
        }
    }
}
