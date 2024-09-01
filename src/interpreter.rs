use {
    crate::{Expression, Literal, Statement, Token, TokenType},
    std::collections::HashMap,
    std::vec::Vec,
};

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct InterpreterError {
    pub token: Option<Token>,
    pub message: String,
}

type ExecuteInterpreterResult = Result<(), InterpreterError>;
type EvaluateInterpreterResult = Result<Literal, InterpreterError>;

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: &Literal) -> Result<(), InterpreterError> {
        let lexeme = &name.lexeme;
        if self.values.contains_key(lexeme) {
            self.values.insert(lexeme.clone(), value.clone());
            return Ok(());
        }

        Err(InterpreterError {
            token: Some(name.clone()),
            message: format!("Undefined variable '{lexeme}'."),
        })
    }

    pub fn get(&self, name: &Token) -> EvaluateInterpreterResult {
        let lexeme = &name.lexeme;
        if let Some(value) = self.values.get(lexeme) {
            return Ok(value.clone());
        }

        Err(InterpreterError {
            token: Some(name.clone()),
            message: format!("Undefined variable '{lexeme}'."),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) -> ExecuteInterpreterResult {
        for statement in statements {
            self.execute(statement)?
        }

        Ok(())
    }

    pub fn execute(&mut self, statement: Statement) -> ExecuteInterpreterResult {
        match statement {
            Statement::Print(expression) => match self.evaluate(expression)? {
                Literal::Number(value) => println!("{value}"),
                value => println!("{value}"),
            },
            Statement::Expression(expression) => {
                self.evaluate(expression)?;
            }
            Statement::Variable { name, initializer } => {
                let mut value = Literal::Nil;
                if let Some(expression) = initializer {
                    value = self.evaluate(expression)?;
                }

                self.environment.define(name.lexeme, value)
            }
            Statement::Block(statements) => {
                self.execute_block(statements, self.environment.clone())?;
            }
        }

        Ok(())
    }

    pub fn execute_block(
        &mut self,
        statements: Vec<Statement>,
        environment: Environment,
    ) -> ExecuteInterpreterResult {
        let previous = self.environment.clone();
        self.environment = environment;

        for statement in statements {
            if let Err(error) = self.execute(statement) {
                self.environment = previous;
                return Err(error);
            }
        }

        self.environment = previous;
        Ok(())
    }

    pub fn evaluate(&mut self, expression: Expression) -> EvaluateInterpreterResult {
        match expression {
            Expression::Literal(literal) => Ok(literal),
            Expression::Grouping(child) => self.evaluate(*child),
            Expression::Unary { operator, right } => {
                let right_child = self.evaluate(*right)?;

                match operator.token_type {
                    TokenType::Bang => Ok(Literal::Boolean(!self.is_truthy(right_child))),
                    TokenType::Minus => Ok(Literal::Number(
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
                let left_child = self.evaluate(*left)?;
                let right_child = self.evaluate(*right)?;

                match operator.token_type {
                    TokenType::Slash => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Literal::Number(x / y));
                    }
                    TokenType::Star => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Literal::Number(x * y));
                    }
                    TokenType::Minus => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Literal::Number(x - y));
                    }
                    TokenType::Plus => {
                        if let (Literal::Number(a), Literal::Number(b)) =
                            (&left_child, &right_child)
                        {
                            return Ok(Literal::Number(*a + *b));
                        }

                        if let (Literal::String(a), Literal::String(b)) =
                            (&left_child, &right_child)
                        {
                            let mut output = a.to_owned();
                            output.push_str(b);

                            return Ok(Literal::String(output));
                        }

                        Err(InterpreterError {
                            token: Some(operator.clone()),
                            message: "Operands must be two numbers or two strings.".into(),
                        })
                    }
                    TokenType::Greater => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Literal::Boolean(x > y));
                    }
                    TokenType::GreaterEqual => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Literal::Boolean(x >= y));
                    }
                    TokenType::Less => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Literal::Boolean(x < y));
                    }
                    TokenType::LessEqual => {
                        let (x, y) =
                            self.check_number_operands(&operator, &left_child, &right_child)?;

                        return Ok(Literal::Boolean(x <= y));
                    }
                    TokenType::BangEqual => Ok(Literal::Boolean(left_child != right_child)),
                    TokenType::EqualEqual => Ok(Literal::Boolean(left_child == right_child)),
                    _ => panic!("unreachable"),
                }
            }
            Expression::Variable(name) => {
                return self.environment.get(&name);
            }
            Expression::Assign { name, right } => {
                let value = self.evaluate(*right)?;

                self.environment.assign(&name, &value)?;

                return Ok(value);
            }
        }
    }

    pub fn is_truthy(&self, literal: Literal) -> bool {
        match literal {
            Literal::Nil => false,
            Literal::Boolean(value) => value,
            _ => true,
        }
    }

    pub fn check_number_operand(
        &self,
        operator: &Token,
        operand: &Literal,
    ) -> Result<f64, InterpreterError> {
        match operand {
            Literal::Number(x) => Ok(*x),
            _ => Err(InterpreterError {
                token: Some(operator.clone()),
                message: "Operand must be a number.".into(),
            }),
        }
    }

    pub fn check_number_operands(
        &self,
        operator: &Token,
        left: &Literal,
        right: &Literal,
    ) -> Result<(f64, f64), InterpreterError> {
        match (left, right) {
            (Literal::Number(x), Literal::Number(y)) => Ok((*x, *y)),
            _ => Err(InterpreterError {
                token: Some(operator.clone()),
                message: "Operands must be a number.".into(),
            }),
        }
    }
}
