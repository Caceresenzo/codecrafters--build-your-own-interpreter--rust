use std::vec::Vec;

use crate::{Expression, FunctionData, Literal, Statement, Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    next_id: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ParseError(String);

type StatementParserResult = Result<Statement, ParseError>;
type ExpressionParserResult = Result<Expression, ParseError>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            next_id: 1,
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        return id;
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements: Vec<Statement> = Vec::new();

        while !self.is_at_end() {
            let statement = self.declaration()?;
            statements.push(statement);
        }

        Ok(statements)
    }

    pub fn declaration(&mut self) -> StatementParserResult {
        if self.match_(&[&TokenType::Class]) {
            return self.class_declaration();
        }

        if self.match_(&[&TokenType::Fun]) {
            return Ok(Statement::Function(self.function("function")?));
        }

        if self.match_(&[&TokenType::Var]) {
            return self.variable();
        }

        self.statement()
    }

    pub fn class_declaration(&mut self) -> StatementParserResult {
        let name = self
            .consume(&TokenType::Identifier, "Expect class name.")?
            .clone();

        let mut superclass: Option<Expression> = None;
        if self.match_(&[&TokenType::Less]) {
            self.consume(&TokenType::Identifier, "Expect superclass name.")?;
            superclass = Some(Expression::Variable {
                id: self.next_id(),
                name: self.previous().clone(),
            })
        }

        self.consume(&TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods: Vec<FunctionData> = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Statement::Class { name, superclass, methods })
    }

    pub fn function(&mut self, kind: &str) -> Result<FunctionData, ParseError> {
        let name = self
            .consume(
                &TokenType::Identifier,
                format!("Expect {kind} name.").as_str(),
            )?
            .clone();

        self.consume(
            &TokenType::LeftParen,
            format!("Expect '(' after {kind} name.").as_str(),
        )?;

        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(self.error(self.peek(), "Can't have more than 255 parameters."));
                }

                parameters.push(
                    self.consume(&TokenType::Identifier, "Expect parameter name.")?
                        .clone(),
                );

                if !self.match_(&[&TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(
            &TokenType::LeftBrace,
            format!("Expect '{{' before {kind} body.").as_str(),
        )?;

        let body = self.block()?;

        Ok(FunctionData {
            name,
            parameters,
            body,
        })
    }

    pub fn statement(&mut self) -> StatementParserResult {
        if self.match_(&[&TokenType::For]) {
            return self.for_();
        }

        if self.match_(&[&TokenType::If]) {
            return self.if_();
        }

        if self.match_(&[&TokenType::Print]) {
            return self.print();
        }

        if self.match_(&[&TokenType::Return]) {
            return self.return_();
        }

        if self.match_(&[&TokenType::While]) {
            return self.while_();
        }

        if self.match_(&[&TokenType::LeftBrace]) {
            return Ok(Statement::Block(self.block()?));
        }

        self.expression_statement()
    }

    pub fn for_(&mut self) -> StatementParserResult {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer: Option<Statement>;
        if self.match_(&[&TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_(&[&TokenType::Var]) {
            initializer = Some(self.variable()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition = Expression::Literal(Literal::Boolean(true));
        if !self.check(&TokenType::Semicolon) {
            condition = self.expression()?;
        }

        self.consume(&TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let mut increment: Option<Expression> = None;
        if !self.check(&TokenType::RightParen) {
            increment = Some(self.expression()?);
        }

        self.consume(&TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(expression) = increment {
            body = Statement::Block(vec![body, Statement::Expression(expression)]);
        }

        body = Statement::While {
            condition,
            body: Box::new(body),
        };

        if let Some(expression) = initializer {
            body = Statement::Block(vec![expression, body]);
        }

        Ok(body)
    }

    pub fn if_(&mut self) -> StatementParserResult {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;

        let mut else_branch: Option<Statement> = None;
        if self.match_(&[&TokenType::Else]) {
            else_branch = Some(self.statement()?);
        }

        Ok(Statement::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    pub fn print(&mut self) -> StatementParserResult {
        let expression = self.expression()?;

        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;

        Ok(Statement::Print(expression))
    }

    pub fn return_(&mut self) -> StatementParserResult {
        let keyword = self.previous().clone();

        let mut value: Option<Expression> = None;
        if !self.check(&TokenType::Semicolon) {
            value = Some(self.expression()?);
        }

        self.consume(&TokenType::Semicolon, "Expect ';' after return value.")?;

        Ok(Statement::Return { keyword, value })
    }

    pub fn while_(&mut self) -> StatementParserResult {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition.")?;

        let body = self.statement()?;

        Ok(Statement::While {
            condition,
            body: Box::new(body),
        })
    }

    pub fn variable(&mut self) -> StatementParserResult {
        let name = self
            .consume(&TokenType::Identifier, "Expect variable name.")?
            .clone();

        let mut initializer: Option<Expression> = None;
        if self.match_(&[&TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Statement::Variable {
            name: name.clone(),
            initializer,
        })
    }

    pub fn block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements: Vec<Statement> = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }

    pub fn expression_statement(&mut self) -> StatementParserResult {
        let expression = self.expression()?;

        self.consume(&TokenType::Semicolon, "Expect ';' after expression.")?;

        Ok(Statement::Expression(expression))
    }

    pub fn expression(&mut self) -> ExpressionParserResult {
        self.assignment()
    }

    pub fn assignment(&mut self) -> ExpressionParserResult {
        let expression = self.or()?;

        if self.match_(&[&TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expression::Variable { id: _, name } = expression {
                return Ok(Expression::Assign {
                    id: self.next_id(),
                    name: name.clone(),
                    right: Box::new(value),
                });
            } else if let Expression::Get { object, name } = expression {
                return Ok(Expression::Set {
                    object,
                    name,
                    value: Box::new(value),
                });
            }

            return Err(self.error(&equals, "Invalid assignment target."));
        }

        Ok(expression)
    }

    pub fn or(&mut self) -> ExpressionParserResult {
        let mut expression = self.and()?;

        while self.match_(&[&TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;

            expression = Expression::Logical {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    pub fn and(&mut self) -> ExpressionParserResult {
        let mut expression = self.equality()?;

        while self.match_(&[&TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;

            expression = Expression::Logical {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    pub fn equality(&mut self) -> ExpressionParserResult {
        let mut expression = self.comparison()?;

        while self.match_(&[&TokenType::BangEqual, &TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expression)
    }

    pub fn comparison(&mut self) -> ExpressionParserResult {
        let mut expression = self.term()?;

        while self.match_(&[
            &TokenType::Greater,
            &TokenType::GreaterEqual,
            &TokenType::Less,
            &TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expression)
    }

    pub fn term(&mut self) -> ExpressionParserResult {
        let mut expression = self.factor()?;

        while self.match_(&[&TokenType::Minus, &TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expression)
    }

    pub fn factor(&mut self) -> ExpressionParserResult {
        let mut expression = self.unary()?;

        while self.match_(&[&TokenType::Slash, &TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expression)
    }

    pub fn unary(&mut self) -> ExpressionParserResult {
        if self.match_(&[&TokenType::Bang, &TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            return Ok(Expression::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    pub fn call(&mut self) -> ExpressionParserResult {
        let mut expression = self.primary()?;

        loop {
            if self.match_(&[&TokenType::LeftParen]) {
                expression = self.finish_call(expression)?
            } else if self.match_(&[&TokenType::Dot]) {
                let name =
                    self.consume(&TokenType::Identifier, "Expect property name after '.'.")?;

                expression = Expression::Get {
                    object: Box::new(expression),
                    name: name.clone(),
                }
            } else {
                break;
            }
        }

        Ok(expression)
    }

    pub fn finish_call(&mut self, callee: Expression) -> ExpressionParserResult {
        let mut arguments: Vec<Expression> = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(self.error(self.peek(), "Can't have more than 255 arguments."));
                }

                arguments.push(self.expression()?);

                if !self.match_(&[&TokenType::Comma]) {
                    break;
                }
            }
        }

        let parenthesis = self.consume(&TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expression::Call {
            callee: Box::new(callee),
            parenthesis: parenthesis.clone(),
            arguments,
        })
    }

    pub fn primary(&mut self) -> ExpressionParserResult {
        if self.match_(&[&TokenType::False]) {
            return Ok(Expression::Literal(Literal::Boolean(false)));
        }

        if self.match_(&[&TokenType::True]) {
            return Ok(Expression::Literal(Literal::Boolean(true)));
        }

        if self.match_(&[&TokenType::Nil]) {
            return Ok(Expression::Literal(Literal::Nil));
        }

        if self.match_(&[&TokenType::Number, &TokenType::String]) {
            return Ok(Expression::Literal(
                self.previous().literal.as_ref().unwrap().clone(),
            ));
        }

        if self.match_(&[&TokenType::This]) {
            return Ok(Expression::This {
                id: self.next_id(),
                keyword: self.previous().clone(),
            });
        }

        if self.match_(&[&TokenType::Identifier]) {
            return Ok(Expression::Variable {
                id: self.next_id(),
                name: self.previous().clone(),
            });
        }

        if self.match_(&[&TokenType::LeftParen]) {
            let expression = self.expression()?;
            self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;

            return Ok(Expression::Grouping(Box::new(expression)));
        }

        Err(self.error(self.peek(), "Expect expression."))
    }

    pub fn match_(&mut self, token_types: &[&TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    pub fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek(), message))
    }

    pub fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == *token_type
    }

    pub fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    pub fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    pub fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    pub fn error(&self, token: &Token, message: &str) -> ParseError {
        let error_message = if token.token_type == TokenType::Eof {
            format!("[line {}] Error at end: {message}", token.line)
        } else {
            format!(
                "[line {}] Error at '{}': {message}",
                token.line, token.lexeme
            )
        };

        return ParseError(error_message);
    }
}
