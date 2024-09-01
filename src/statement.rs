use crate::{Expression, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Variable {
        name: Token,
        initializer: Option<Expression>,
    },
}
