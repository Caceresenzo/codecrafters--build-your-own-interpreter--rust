use std::collections::{HashMap, VecDeque};

use crate::{Expression, FunctionData, Interpreter, Statement, Token};

#[derive(Debug, PartialEq, Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ClassType {
    None,
    Class,
}

#[derive(Debug)]
pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: VecDeque<HashMap<String, bool>>,
    current_function_type: FunctionType,
    current_class_type: ClassType,
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
            current_class_type: ClassType::None,
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
                    if self.current_function_type == FunctionType::Initializer {
                        return Err(ResolverError {
                            token: keyword.clone(),
                            message: "Can't return a value from an initializer.".into(),
                        });
                    }

                    self.resolve_expression(expression)?;
                }

                Ok(())
            }

            Statement::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;

                Ok(())
            }

            Statement::Class {
                name,
                superclass,
                methods,
            } => {
                let enclosing_type = self.current_class_type;
                self.current_class_type = ClassType::Class;

                self.declare(name)?;
                self.define(name);

                if superclass.is_some() {
                    if let Expression::Variable {
                        id: _,
                        name: superclass_name,
                    } = superclass.as_ref().unwrap()
                    {
                        if name.lexeme.eq(&superclass_name.lexeme) {
                            return Err(ResolverError {
                                token: superclass_name.clone(),
                                message: "A class can't inherit from itself.".into(),
                            });
                        }
                    } else {
                        panic!();
                    }

                    self.resolve_expression(superclass.as_ref().unwrap())?;

                    self.begin_scope();
                    self.scopes.back_mut().unwrap().insert("super".into(), true);
                }

                self.begin_scope();
                self.scopes.back_mut().unwrap().insert("this".into(), true);

                for method in methods {
                    let declaration = if method.name.lexeme.eq("init") {
                        FunctionType::Initializer
                    } else {
                        FunctionType::Method
                    };

                    self.resolve_function(method, declaration)?;
                }

                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }

                self.current_class_type = enclosing_type;

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

            Expression::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expression(value)?;
                self.resolve_expression(object)?;

                Ok(())
            }

            Expression::This { id, keyword } => {
                if self.current_class_type == ClassType::None {
                    return Err(ResolverError {
                        token: keyword.clone(),
                        message: "Can't use 'this' outside of a class.".into(),
                    });
                }

                self.resolve_local(*id, keyword);

                Ok(())
            }

            Expression::Super {
                id,
                keyword,
                method: _,
            } => {
                self.resolve_local(id.clone(), keyword);

                Ok(())
            }
        };
    }
}
