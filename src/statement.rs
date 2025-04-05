use {
    crate::{Expression, Token},
    std::vec::Vec,
};

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionData {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    Function(FunctionData),
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
    Class {
        name: Token,
        superclass: Option<Expression>,
        methods: Vec<FunctionData>,
    },
}
