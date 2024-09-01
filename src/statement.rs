use {
    crate::{Expression, Token},
    std::vec::Vec,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Variable {
        name: Token,
        initializer: Option<Expression>,
    },
    Block(Vec<Statement>),
}
