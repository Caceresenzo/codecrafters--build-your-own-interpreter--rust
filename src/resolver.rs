use std::collections::{HashMap, VecDeque};

use crate::{Expression, Interpreter, Statement, Token};

#[derive(Debug)]
pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: VecDeque<HashMap<String, bool>>,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ResolverError {
    pub token: Token,
    pub message: String,
}

pub type ResolverResult = Result<(), ResolverError>;

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: VecDeque::new(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push_back(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop_back();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.back_mut() {
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.back_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, expression: &Expression, name: &Token) {
        println!("resolve_local {}", self.scopes.len());

        for (index, scope) in self.scopes.iter().enumerate().rev() {
            println!("{index} {scope:?}");
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(expression, (self.scopes.len() - 1 - index) as u32);
                return;
            }
        }
    }

    fn resolve_function(&mut self, function: &Statement) -> ResolverResult {
        if let Statement::Function {
            name: _,
            parameters,
            body,
        } = function
        {
            self.begin_scope();

            for parameter in parameters {
                self.declare(parameter);
                self.define(parameter);
            }

            self.resolve_statements(body)?;

            self.end_scope();
        } else {
            panic!("statement must be a function");
        }

        Ok(())
    }

    pub fn resolve_statements(&mut self, statements: &Vec<Statement>) -> ResolverResult {
        for statement in statements {
            self.resolve_statement(statement)?
        }

        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Statement) -> ResolverResult {
        match statement {
            Statement::Block(statements) => {
                self.begin_scope();

                self.resolve_statements(statements)?;

                self.end_scope();
            }

            Statement::Variable { name, initializer } => {
                self.declare(&name);

                if let Some(expression) = initializer {
                    self.resolve_expression(expression)?
                }

                self.define(&name);
            }

            Statement::Function {
                name,
                parameters: _,
                body: _,
            } => {
                self.declare(name);
                self.define(name);

                self.resolve_function(statement)?;
            }

            Statement::Expression(expression) => {
                self.resolve_expression(expression)?;
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;

                if let Some(else_branch_expression) = else_branch {
                    self.resolve_statement(else_branch_expression)?
                }
            }

            Statement::Print(expression) => {
                self.resolve_expression(expression)?;
            }

            Statement::Return { keyword: _, value } => {
                if let Some(expression) = value {
                    self.resolve_expression(expression)?;
                }
            }

            Statement::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;
            }
        }

        Ok(())
    }

    fn resolve_expression(&mut self, expression: &Expression) -> ResolverResult {
        match expression {
            Expression::Variable(name) => {
                // if !self.scopes.is_empty()
                //     && self.scopes.back().unwrap().get(&name.lexeme) == Some(&true)
                // {
                //     return Err(ResolverError {
                //         token: name.clone(),
                //         message: "Can't read local variable in its own initializer.".into(),
                //     });
                // }

                self.resolve_local(expression, name);
            }

            Expression::Assign { name, right } => {
                self.resolve_expression(right)?;
                self.resolve_local(expression, name);
            }

            Expression::Binary { left, operator: _, right } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }

            Expression::Call { callee, parenthesis: _, arguments } => {
                self.resolve_expression(&callee)?;

                for argument in arguments {
                    self.resolve_expression(argument)?;
                }
            }

            Expression::Grouping(expression) => {
                self.resolve_expression(expression)?;
            }

            Expression::Literal(_) => {}

            Expression::Logical { left, operator: _, right } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }

            Expression::Unary { operator: _, right } => {
                self.resolve_expression(right)?
            }
        }

        Ok(())
    }
}
