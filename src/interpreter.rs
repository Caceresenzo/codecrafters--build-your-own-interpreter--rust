use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    native, Callable, Class, Environment, Expression, Instance, LoxFunction, Statement, Token,
    TokenType, Value,
};

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct InterpreterError {
    pub token: Option<Token>,
    pub message: String,
}

pub type ExecuteInterpreterResult = Result<Option<Value>, InterpreterError>;
pub type EvaluateInterpreterResult = Result<Value, InterpreterError>;

#[derive(Debug)]
pub struct Interpreter {
    pub globals: Environment,
    pub environment: Environment,
    locals: HashMap<u64, u32>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut environment = Environment::new();

        environment.define(
            "clock".into(),
            Value::Function(Rc::new(RefCell::new(native::ClockFunction {}))),
        );

        Interpreter {
            globals: environment.clone(),
            environment,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Statement>) -> ExecuteInterpreterResult {
        for statement in statements {
            self.execute(statement)?;
        }

        Ok(None)
    }

    pub fn execute(&mut self, statement: &Statement) -> ExecuteInterpreterResult {
        match statement {
            Statement::Expression(expression) => {
                self.evaluate(expression)?;

                Ok(None)
            }
            Statement::Function(data) => {
                let function = LoxFunction::new(data, false, self.environment.clone());

                self.environment.define(
                    function.get_name().into(),
                    Value::Function(Rc::new(RefCell::new(function))),
                );

                Ok(None)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let result = self.evaluate(condition)?;

                if self.is_truthy(result) {
                    Ok(self.execute(then_branch)?)
                } else if let Some(statement) = else_branch {
                    Ok(self.execute(statement)?)
                } else {
                    Ok(None)
                }
            }
            Statement::Print(expression) => {
                match self.evaluate(expression)? {
                    Value::Number(value) => println!("{value}"),
                    value => println!("{value}"),
                }

                Ok(None)
            }
            Statement::Variable { name, initializer } => {
                let mut value = Value::Nil;
                if let Some(expression) = initializer {
                    value = self.evaluate(expression)?;
                }

                self.environment.define(name.lexeme.clone(), value);

                Ok(None)
            }
            Statement::Return { keyword: _, value } => {
                if let Some(expression) = value {
                    return Ok(Some(self.evaluate(expression)?));
                }

                Ok(Some(Value::Nil))
            }
            Statement::While { condition, body } => {
                loop {
                    let is_true = self.evaluate(condition)?;

                    if !self.is_truthy(is_true) {
                        break;
                    }

                    if let Some(returned) = self.execute(body)? {
                        return Ok(Some(returned));
                    }
                }

                Ok(None)
            }
            Statement::Block(statements) => {
                Ok(self.execute_block(statements, self.environment.enclose())?)
            }

            Statement::Class {
                name,
                superclass,
                methods,
            } => {
                let mut superclass_rc: Option<Rc<RefCell<Class>>> = None;
                if let Some(superclass_expression) = superclass {
                    let result = self.evaluate(superclass_expression)?;

                    if let Value::Class(rc) = result {
                        superclass_rc = Some(rc);
                    } else {
                        return Err(InterpreterError {
                            token: Some(name.clone()),
                            message: "Superclass must be a class.".into(),
                        });
                    }
                }

                self.environment.define(name.lexeme.clone(), Value::Nil);

                if superclass.is_some() {
                    self.environment = self.environment.enclose();
                    self.environment
                        .define("super".into(), Value::Class(superclass_rc.clone().unwrap()));
                }

                let mut loaded_methods: HashMap<String, Rc<RefCell<LoxFunction>>> = HashMap::new();
                for method in methods {
                    let function = LoxFunction::new(
                        method,
                        method.name.lexeme.eq("init"),
                        self.environment.clone(),
                    );

                    loaded_methods
                        .insert(method.name.lexeme.clone(), Rc::new(RefCell::new(function)));
                }

                let class = Class::new(name.lexeme.clone(), superclass_rc, loaded_methods);

                if superclass.is_some() {
                    self.environment = self.environment.enclosing();
                }

                self.environment.define(
                    name.lexeme.clone(),
                    Value::Class(Rc::new(RefCell::new(class))),
                );

                Ok(None)
            }
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Statement>,
        environment: Environment,
    ) -> ExecuteInterpreterResult {
        let previous = self.environment.clone();
        self.environment = environment;

        for statement in statements {
            match self.execute(statement) {
                Err(error) => {
                    self.environment = previous;
                    return Err(error);
                }
                Ok(Some(value)) => {
                    self.environment = previous;
                    return Ok(Some(value));
                }
                Ok(None) => {}
            }
        }

        self.environment = previous;
        Ok(None)
    }

    pub fn evaluate(&mut self, expression: &Expression) -> EvaluateInterpreterResult {
        match expression {
            Expression::Literal(literal) => Ok(literal.clone().into()),
            Expression::Grouping(child) => self.evaluate(child),
            Expression::Unary { operator, right } => {
                let right_child = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Bang => Ok(Value::Boolean(!self.is_truthy(right_child))),
                    TokenType::Minus => Ok(Value::Number(
                        -self.check_number_operand(&operator, &right_child)?,
                    )),
                    _ => panic!("unreachable"),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_child = self.evaluate(left)?;
                let right_child = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Slash => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Value::Number(x / y));
                    }
                    TokenType::Star => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Value::Number(x * y));
                    }
                    TokenType::Minus => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Value::Number(x - y));
                    }
                    TokenType::Plus => {
                        if let (Value::Number(a), Value::Number(b)) = (&left_child, &right_child) {
                            return Ok(Value::Number(*a + *b));
                        }

                        if let (Value::String(a), Value::String(b)) = (&left_child, &right_child) {
                            let mut output: String = a.as_str().into();
                            output.push_str(b);

                            return Ok(Value::String(Rc::new(output)));
                        }

                        Err(InterpreterError {
                            token: Some(operator.clone()),
                            message: "Operands must be two numbers or two strings.".into(),
                        })
                    }
                    TokenType::Greater => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Value::Boolean(x > y));
                    }
                    TokenType::GreaterEqual => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Value::Boolean(x >= y));
                    }
                    TokenType::Less => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Value::Boolean(x < y));
                    }
                    TokenType::LessEqual => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Value::Boolean(x <= y));
                    }
                    TokenType::BangEqual => Ok(Value::Boolean(left_child != right_child)),
                    TokenType::EqualEqual => Ok(Value::Boolean(left_child == right_child)),
                    _ => panic!("unreachable"),
                }
            }
            Expression::Variable { id, name } => {
                return self.look_up_variable(&name, *id);
            }
            Expression::Assign { id, name, right } => {
                let value = self.evaluate(right)?;

                if let Some(distance) = self.locals.get(id) {
                    self.environment.assign_at(*distance, name, &value)?;
                } else {
                    self.globals.assign(name, &value)?;
                }

                return Ok(value);
            }
            Expression::Logical {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left)?;
                let is_left_truthy = self.is_truthy(left_value.clone());

                match operator.token_type {
                    TokenType::Or => {
                        if is_left_truthy {
                            return Ok(left_value);
                        }

                        self.evaluate(right)
                    }
                    TokenType::And => {
                        if !is_left_truthy {
                            return Ok(left_value);
                        }

                        self.evaluate(right)
                    }
                    _ => panic!("unreachable"),
                }
            }
            Expression::Call {
                callee,
                parenthesis,
                arguments,
            } => {
                let callee_value = self.evaluate(callee)?;

                let mut arguments_values: Vec<Value> = Vec::new();
                for argument in arguments {
                    let argument_value = self.evaluate(argument)?;
                    arguments_values.push(argument_value);
                }

                match callee_value {
                    Value::Function(callable) => {
                        let arity = callable.borrow().arity();
                        if arguments_values.len() != arity {
                            return Err(InterpreterError {
                                token: Some(parenthesis.clone()),
                                message: format!(
                                    "Expected {arity} arguments but got {}.",
                                    arguments_values.len()
                                ),
                            });
                        }

                        let returned_value =
                            callable
                                .borrow()
                                .call(self, arguments_values, parenthesis)?;

                        Ok(returned_value.unwrap_or(Value::Nil))
                    }

                    Value::Class(class) => {
                        let instance = Instance::new(class.clone());
                        let instance_rc = Rc::new(RefCell::new(instance));

                        if let Some(initializer) = class.borrow().find_function("init".into()) {
                            let arity = initializer.borrow().arity();
                            if arguments_values.len() != arity {
                                return Err(InterpreterError {
                                    token: Some(parenthesis.clone()),
                                    message: format!(
                                        "Expected {arity} arguments but got {}.",
                                        arguments_values.len()
                                    ),
                                });
                            }

                            initializer.borrow().bind(instance_rc.clone()).call(
                                self,
                                arguments_values,
                                parenthesis,
                            )?;
                        }

                        Ok(Value::Instance(instance_rc))
                    }

                    _ => Err(InterpreterError {
                        token: Some(parenthesis.clone()),
                        message: "Can only call functions and classes.".into(),
                    }),
                }
            }

            Expression::Get { object, name } => {
                let object_value = self.evaluate(object)?;
                if let Value::Instance(instance) = &object_value {
                    return instance.borrow().get(name, instance.clone());
                }

                Err(InterpreterError {
                    token: Some(name.clone()),
                    message: "Only instances have properties.".into(),
                })
            }

            Expression::Set {
                object,
                name,
                value,
            } => {
                if let Value::Instance(instance) = self.evaluate(object)? {
                    let evaluated_value = self.evaluate(value)?;

                    instance.borrow_mut().set(name, evaluated_value.clone())?;

                    return Ok(evaluated_value);
                }

                Err(InterpreterError {
                    token: Some(name.clone()),
                    message: "Only instances have fields.".into(),
                })
            }

            Expression::This { id, keyword } => self.look_up_variable(keyword, *id),

            Expression::Super {
                id,
                keyword: _,
                method,
            } => {
                if let Some(distance) = self.locals.get(id) {
                    if let Value::Class(superclass) = self
                        .environment
                        .get_at(distance.clone(), "super".into())
                        .unwrap()
                    {
                        if let Value::Instance(instance) = self
                            .environment
                            .get_at(distance.clone() - 1, "this".into())
                            .unwrap()
                        {
                            if let Some(method) =
                                superclass.borrow().find_function(method.lexeme.clone())
                            {
                                return Ok(Value::Function(Rc::new(RefCell::new(
                                    method.borrow().bind(instance),
                                ))));
                            } else {
                                return Err(InterpreterError {
                                    token: Some(method.clone()),
                                    message: format!("Undefined property '{}'.", method.lexeme),
                                });
                            }
                        } else {
                            panic!();
                        }
                    } else {
                        panic!();
                    }
                }

                return Ok(Value::Nil);
            }
        }
    }

    pub fn resolve(&mut self, expression_id: u64, depth: u32) {
        self.locals.insert(expression_id, depth);
    }

    pub fn look_up_variable(
        &mut self,
        name: &Token,
        expression_id: u64,
    ) -> EvaluateInterpreterResult {
        if let Some(distance) = self.locals.get(&expression_id) {
            // println!("{} {expression_id} found at distance {}", name.lexeme, *distance);
            self.environment.get_at(*distance, name.lexeme.clone())
        } else {
            // println!("{} {expression_id} not found", name.lexeme);
            self.globals.get(name)
        }
    }

    pub fn is_truthy(&self, value: Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(value) => value,
            _ => true,
        }
    }

    pub fn check_number_operand(
        &self,
        operator: &Token,
        operand: &Value,
    ) -> Result<f64, InterpreterError> {
        match operand {
            Value::Number(x) => Ok(*x),
            _ => Err(InterpreterError {
                token: Some(operator.clone()),
                message: "Operand must be a number.".into(),
            }),
        }
    }

    pub fn check_number_operands(
        &self,
        operator: &Token,
        left: &Value,
        right: &Value,
    ) -> Result<(f64, f64), InterpreterError> {
        match (left, right) {
            (Value::Number(x), Value::Number(y)) => Ok((*x, *y)),
            _ => Err(InterpreterError {
                token: Some(operator.clone()),
                message: "Operands must be a number.".into(),
            }),
        }
    }
}
