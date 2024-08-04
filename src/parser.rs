use crate::{Expression, Literal, Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn expression(&mut self) -> Expression {
        self.primary()
    }

    pub fn primary(&mut self) -> Expression {
        if self.match_(&[&TokenType::False]) {
            return Expression::Literal(Literal::Boolean(false));
        }

        if self.match_(&[&TokenType::True]) {
            return Expression::Literal(Literal::Boolean(true));
        }

        if self.match_(&[&TokenType::Nil]) {
            return Expression::Literal(Literal::Nil);
        }

        if self.match_(&[&TokenType::Number, &TokenType::String]) {
            return Expression::Literal(self.previous().literal.as_ref().unwrap().clone());
        }

        panic!();
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
}
