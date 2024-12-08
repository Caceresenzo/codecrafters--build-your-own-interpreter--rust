use {
    crate::{Expression, Token},
    std::vec::Vec,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    Function {
        name: Token,
        parameters: Vec<Token>,
        body: Vec<Statement>,
    },
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
    Return {
        keyword: Token,
        value: Option<Expression>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    Block(Vec<Statement>),
}
