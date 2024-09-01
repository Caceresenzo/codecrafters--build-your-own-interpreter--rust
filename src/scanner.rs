use std::collections::HashMap;

use crate::{Literal, Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Scanner {
    source: String,
    source_len: usize,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    pub had_error: bool,
    keywords: HashMap<&'static str, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source_len: source.chars().count(),
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            had_error: false,
            keywords: HashMap::from([
                ("and", TokenType::And),
                ("class", TokenType::Class),
                ("else", TokenType::Else),
                ("false", TokenType::False),
                ("for", TokenType::For),
                ("fun", TokenType::Fun),
                ("if", TokenType::If),
                ("nil", TokenType::Nil),
                ("or", TokenType::Or),
                ("print", TokenType::Print),
                ("return", TokenType::Return),
                ("super", TokenType::Super),
                ("this", TokenType::This),
                ("true", TokenType::True),
                ("var", TokenType::Var),
                ("while", TokenType::While),
            ]),
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source_len
    }

    pub fn text(&self) -> String {
        self.source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect::<String>()
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "".into(), None, self.line));

        self.tokens.clone()
    }

    pub fn scan_token(&mut self) {
        let character = self.advance();

        match character {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '=' if self.match_('=') => self.add_token(TokenType::EqualEqual, None),
            '=' => self.add_token(TokenType::Equal, None),
            '!' if self.match_('=') => self.add_token(TokenType::BangEqual, None),
            '!' => self.add_token(TokenType::Bang, None),
            '<' if self.match_('=') => self.add_token(TokenType::LessEqual, None),
            '<' => self.add_token(TokenType::Less, None),
            '>' if self.match_('=') => self.add_token(TokenType::GreaterEqual, None),
            '>' => self.add_token(TokenType::Greater, None),
            '/' if self.match_('/') => self.advance_next_line(),
            '/' => self.add_token(TokenType::Slash, None),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => {
                if self.is_number(character) {
                    self.number()
                } else if self.is_alpha_or_number(character) {
                    self.identifier()
                } else {
                    self.error(self.line, format!("Unexpected character: {}", character))
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let index = self.current;
        self.current += 1;
        self.source.chars().nth(index).unwrap()
    }

    fn advance_next_line(&mut self) {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
    }

    fn peek(&self) -> char {
        self.peek_at(0)
    }

    fn peek_at(&self, n: usize) -> char {
        let index = self.current + n;

        if index >= self.source_len {
            return '\0';
        }

        self.source.chars().nth(index).unwrap()
    }

    fn match_(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        self.tokens
            .push(Token::new(token_type, self.text(), literal, self.line));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            self.error(self.line, "Unterminated string.".into());
            return;
        }

        // closing "
        self.advance();

        let value = self
            .source
            .chars()
            .skip(self.start + 1)
            .take(self.current - self.start - 2)
            .collect::<String>();

        self.add_token(TokenType::String, Some(Literal::String(value.into())))
    }

    fn number(&mut self) {
        while self.is_number(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_number(self.peek_at(1)) {
            // consume .
            self.advance();

            while self.is_number(self.peek()) {
                self.advance();
            }
        }

        let value: f64 = self.text().parse().unwrap();
        self.add_token(TokenType::Number, Some(Literal::Number(value)));
    }

    fn identifier(&mut self) {
        while self.is_alpha_or_number(self.peek()) {
            self.advance();
        }

        self.add_token(
            self.keywords
                .get(self.text().as_str())
                .unwrap_or(&TokenType::Identifier)
                .clone(),
            None,
        );
    }

    fn is_number(&self, character: char) -> bool {
        return character.is_numeric();
    }

    fn is_alpha(&self, character: char) -> bool {
        return character.is_alphabetic() || character == '_';
    }

    fn is_alpha_or_number(&self, character: char) -> bool {
        return self.is_alpha(character) || self.is_number(character);
    }

    fn error(&mut self, line: usize, message: String) {
        self.report(line, "".into(), message);
    }

    fn report(&mut self, line: usize, where_: String, message: String) {
        eprintln!("[line {}] Error{}: {}", line, where_, message);
        self.had_error = true;
    }
}
