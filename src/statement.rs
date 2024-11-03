use {
    crate::{Expression, Token},
    std::vec::Vec,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Print(Expression),
    Variable {
        name: Token,
        initializer: Option<Expression>,
    },
    Block(Vec<Statement>),
}
