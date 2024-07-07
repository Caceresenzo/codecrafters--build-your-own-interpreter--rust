use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    StringLiteral(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Other.
    Eof,
}

// Implement the fmt::Display trait for TokenType
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "LEFT_PAREN"),
            TokenType::RightParen => write!(f, "RIGHT_PAREN"),
            TokenType::LeftBrace => write!(f, "LEFT_BRACE"),
            TokenType::RightBrace => write!(f, "RIGHT_BRACE"),
            TokenType::Comma => write!(f, "COMMA"),
            TokenType::Dot => write!(f, "DOT"),
            TokenType::Minus => write!(f, "MINUS"),
            TokenType::Plus => write!(f, "PLUS"),
            TokenType::Semicolon => write!(f, "SEMICOLON"),
            TokenType::Slash => write!(f, "SLASH"),
            TokenType::Star => write!(f, "STAR"),
            TokenType::Bang => write!(f, "BANG"),
            TokenType::BangEqual => write!(f, "BANG_EQUAL"),
            TokenType::Equal => write!(f, "EQUAL"),
            TokenType::EqualEqual => write!(f, "EQUAL_EQUAL"),
            TokenType::Greater => write!(f, "GREATER"),
            TokenType::GreaterEqual => write!(f, "GREATER_EQUAL"),
            TokenType::Less => write!(f, "LESS"),
            TokenType::LessEqual => write!(f, "LESS_EQUAL"),
            TokenType::Identifier(_) => write!(f, "IDENTIFIER"),
            TokenType::StringLiteral(_) => write!(f, "STRING"),
            TokenType::Number(_) => write!(f, "NUMBER"),
            TokenType::And => write!(f, "AND"),
            TokenType::Class => write!(f, "CLASS"),
            TokenType::Else => write!(f, "ELSE"),
            TokenType::False => write!(f, "FALSE"),
            TokenType::Fun => write!(f, "FUN"),
            TokenType::For => write!(f, "FOR"),
            TokenType::If => write!(f, "IF"),
            TokenType::Nil => write!(f, "NIL"),
            TokenType::Or => write!(f, "OR"),
            TokenType::Print => write!(f, "PRINT"),
            TokenType::Return => write!(f, "RETURN"),
            TokenType::Super => write!(f, "SUPER"),
            TokenType::This => write!(f, "THIS"),
            TokenType::True => write!(f, "TRUE"),
            TokenType::Var => write!(f, "VAR"),
            TokenType::While => write!(f, "WHILE"),
            TokenType::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let literal: String = match &self.token_type {
            TokenType::Identifier(value) => value.into(),
            TokenType::StringLiteral(value) => value.into(),
            TokenType::Number(value) => value.to_string(),
            _ => "null".into(),
        };

        write!(f, "{} {} {}", self.token_type, self.lexeme, literal)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Scanner {
    source: String,
    source_len: usize,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    had_error: bool,
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
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source_len
    }

    pub fn text(&self) -> String {
        self.source[self.start..self.current].into()
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "".into(), self.line));

        self.tokens.clone()
    }

    pub fn scan_token(&mut self) {
        let character = self.advance();

        match character {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '=' if self.match_('=') => self.add_token(TokenType::EqualEqual),
            '=' => self.add_token(TokenType::Equal),
            '!' if self.match_('=') => self.add_token(TokenType::BangEqual),
            '!' => self.add_token(TokenType::Bang),
            '<' if self.match_('=') => self.add_token(TokenType::LessEqual),
            '<' => self.add_token(TokenType::Less),
            '>' if self.match_('=') => self.add_token(TokenType::GreaterEqual),
            '>' => self.add_token(TokenType::Greater),
            '/' if self.match_('/') => self.advance_next_line(),
            '/' => self.add_token(TokenType::Slash),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            _ => self.error(self.line, format!("Unexpected character: {}", character)),
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

    fn add_token(&mut self, token_type: TokenType) {
        self.tokens
            .push(Token::new(token_type, self.text(), self.line));
    }

    fn error(&mut self, line: usize, message: String) {
        self.report(line, "".into(), message);
    }

    fn report(&mut self, line: usize, where_: String, message: String) {
        eprintln!("[line {}] Error{}: {}", line, where_, message);
        self.had_error = true;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            let mut scanner = Scanner::new(file_contents);
            let tokens = scanner.scan_tokens();

            for token in tokens {
                println!("{}", token);
            }

            if scanner.had_error {
                exit(65);
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
