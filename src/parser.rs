use crate::{Expression, Literal, Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ParseError(String);

type ParserResult = Result<Expression, ParseError>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn expression(&mut self) -> ParserResult {
        self.primary()
    }

    pub fn primary(&mut self) -> ParserResult {
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
