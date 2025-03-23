use std::collections::{HashMap, VecDeque};

use crate::{Expression, FunctionData, Interpreter, Statement, Token};

#[derive(Debug, PartialEq, Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Method,
}

#[derive(Debug)]
pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: VecDeque<HashMap<String, bool>>,
    current_function_type: FunctionType,
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
            current_function_type: FunctionType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push_back(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop_back();
    }

    fn declare(&mut self, name: &Token) -> ResolverResult {
        if let Some(scope) = self.scopes.back_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err(ResolverError {
                    token: name.clone(),
                    message: "Already a variable with this name in this scope.".into(),
                });
            }

            scope.insert(name.lexeme.clone(), false);
        }

        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.back_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, expression_id: u64, name: &Token) {
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                // println!("resolve {} ({expression_id}) at distance {}", name.lexeme, self.scopes.len() - 1 - index);
                self.interpreter
                    .resolve(expression_id, (self.scopes.len() - 1 - index) as u32);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &FunctionData,
        function_type: FunctionType,
    ) -> ResolverResult {
        let enclosing_type = self.current_function_type;
        self.current_function_type = function_type;

        self.begin_scope();

        for parameter in &function.parameters {
            self.declare(parameter)?;
            self.define(parameter);
        }

        self.resolve_statements(&function.body)?;

        self.end_scope();

        self.current_function_type = enclosing_type;

        Ok(())
    }

    pub fn resolve_statements(&mut self, statements: &Vec<Statement>) -> ResolverResult {
        for statement in statements {
            self.resolve_statement(statement)?
        }

        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Statement) -> ResolverResult {
        return match statement {
            Statement::Block(statements) => {
                self.begin_scope();

                self.resolve_statements(statements)?;

                self.end_scope();

                Ok(())
            }

            Statement::Variable { name, initializer } => {
                self.declare(&name)?;

                if let Some(expression) = initializer {
                    self.resolve_expression(expression)?
                }

                self.define(&name);

                Ok(())
            }

            Statement::Function(data) => {
                self.declare(&data.name)?;
                self.define(&data.name);

                self.resolve_function(data, FunctionType::Function)?;

                Ok(())
            }

            Statement::Expression(expression) => {
                self.resolve_expression(expression)?;

                Ok(())
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

                Ok(())
            }

            Statement::Print(expression) => {
                self.resolve_expression(expression)?;

                Ok(())
            }

            Statement::Return { keyword, value } => {
                if self.current_function_type == FunctionType::None {
                    return Err(ResolverError {
                        token: keyword.clone(),
                        message: "Can't return from top-level code.".into(),
                    });
                }

                if let Some(expression) = value {
                    self.resolve_expression(expression)?;
                }

                Ok(())
            }

            Statement::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;

                Ok(())
            }

            Statement::Class { name, methods } => {
                self.declare(name)?;
                self.define(name);

                for method in methods {
                    let declaration = FunctionType::Method;

                    self.resolve_function(method, declaration)?;
                }

                Ok(())
            }
        };
    }

    fn resolve_expression(&mut self, expression: &Expression) -> ResolverResult {
        return match expression {
            Expression::Variable { id, name } => {
                if !self.scopes.is_empty()
                    && self.scopes.back().unwrap().get(&name.lexeme) == Some(&false)
                {
                    return Err(ResolverError {
                        token: name.clone(),
                        message: "Can't read local variable in its own initializer.".into(),
                    });
                }

                self.resolve_local(*id, name);

                Ok(())
            }

            Expression::Assign { id, name, right } => {
                self.resolve_expression(right)?;
                self.resolve_local(*id, name);

                Ok(())
            }

            Expression::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;

                Ok(())
            }

            Expression::Call {
                callee,
                parenthesis: _,
                arguments,
            } => {
                self.resolve_expression(&callee)?;

                for argument in arguments {
                    self.resolve_expression(argument)?;
                }

                Ok(())
            }

            Expression::Grouping(expression) => {
                self.resolve_expression(expression)?;

                Ok(())
            }

            Expression::Literal(_) => Ok(()),

            Expression::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;

                Ok(())
            }

            Expression::Unary { operator: _, right } => {
                self.resolve_expression(right)?;

                Ok(())
            }

            Expression::Get { object, name: _ } => {
                self.resolve_expression(object)?;

                Ok(())
            }

            Expression::Set { object, name: _, value } => {
                self.resolve_expression(value)?;
                self.resolve_expression(object)?;

                Ok(())
            }
        };
    }
}
