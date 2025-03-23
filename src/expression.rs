use std::fmt;

use crate::{Literal, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Grouping(Box<Expression>),
    Unary {
        operator: Token,
        right: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Variable {
        id: u64,
        name: Token,
    },
    Assign {
        id: u64,
        name: Token,
        right: Box<Expression>,
    },
    Logical {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        parenthesis: Token,
        arguments: Vec<Expression>,
    },
    Get {
        object: Box<Expression>,
        name: Token,
    },
    Set {
        object: Box<Expression>,
        name: Token,
        value: Box<Expression>,
    },
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(value) => write!(f, "{value}"),
            Expression::Grouping(expression) => write!(f, "(group {expression})"),
            Expression::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme),
            Expression::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {left} {right})", operator.lexeme),
            Expression::Variable { id: _, name } => write!(f, "(var {})", name.lexeme),
            Expression::Assign { id: _, name, right } => {
                write!(f, "(assign {} {right})", name.lexeme)
            }
            Expression::Logical {
                left,
                operator,
                right,
            } => write!(f, "(logical {} {left} {right})", operator.lexeme),
            Expression::Call {
                callee,
                parenthesis,
                arguments,
            } => write!(f, "(call {callee} {parenthesis} {arguments:?})"),
            _ => todo!(),
        }
    }
}
