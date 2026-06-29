use crate::token::{Token, TokenType};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    // Вспомогательные методы
    fn peek(&self) -> Option<&char> {
        self.input.get(self.pos)
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).cloned();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    // Главная функция получения следующего токена
    pub fn next_token(&mut self) -> Token {
        // Скипаем пробелы
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }

        let line = self.line;
        let col = self.column;

        match self.peek() {
            Some(&c) => {
                match c {
                    // Односимвольные токены
                    '%' => { self.advance(); Token { kind: TokenType::Percent, line, column: col } },
                    '+' => { self.advance(); Token { kind: TokenType::Plus, line, column: col } },
                    '*' => { self.advance(); Token { kind: TokenType::Star, line, column: col } },
                    ',' => { self.advance(); Token { kind: TokenType::Comma, line, column: col } },
                    ';' => { self.advance(); Token { kind: TokenType::Semi, line, column: col } },
                    ':' => { 
                        self.advance(); 
                        // Проверка на :: (ColonColon)
                        if self.peek() == Some(&':') {
                            self.advance();
                            Token { kind: TokenType::ColonColon, line, column: col }
                        } else {
                            Token { kind: TokenType::Colon, line, column: col }
                        }
                    },
                    '!' => {
                        self.advance();
                        if self.peek() == Some(&'!') {
                            self.advance();
                            Token { kind: TokenType::DoubleBang, line, column: col }
                        } else if self.peek() == Some(&'=') {
                            self.advance();
                            Token { kind: TokenType::BangEq, line, column: col }
                        } else {
                            Token { kind: TokenType::Bang, line, column: col }
                        }
                    },
                    '.' => {
                        self.advance();
                        if self.peek() == Some(&'.') {
                            self.advance();
                            Token { kind: TokenType::DotDot, line, column: col }
                        } else {
                            Token { kind: TokenType::Dot, line, column: col }
                        }
                    },
                    '?' => {
                        self.advance();
                        if self.peek() == Some(&'?') {
                            self.advance();
                            Token { kind: TokenType::DoubleQuestion, line, column: col }
                        } else {
                            Token { kind: TokenType::Unknown('?'), line, column: col }
                        }
                    },
                    '<' => {
                        self.advance();
                        if self.peek() == Some(&'=') {
                            self.advance();
                            Token { kind: TokenType::LtEq, line, column: col }
                        } else {
                            Token { kind: TokenType::Lt, line, column: col }
                        }
                    },
                    '>' => {
                        self.advance();
                        if self.peek() == Some(&'=') {
                            self.advance();
                            Token { kind: TokenType::GtEq, line, column: col }
                        } else {
                            Token { kind: TokenType::Gt, line, column: col }
                        }
                    },
                    '-' => {
                        self.advance();
                        if self.peek() == Some(&'>') {
                            self.advance();
                            Token { kind: TokenType::Arrow, line, column: col }
                        } else {
                            Token { kind: TokenType::Minus, line, column: col }
                        }
                    },
                    '=' => {
                        self.advance();
                        if self.peek() == Some(&'=') {
                            self.advance();
                            Token { kind: TokenType::EqEq, line, column: col }
                        } else {
                            Token { kind: TokenType::Eq, line, column: col }
                        }
                    },
                    '/' => {
                        self.advance();
                        if self.peek() == Some(&'/') {
                            while let Some(&ch) = self.peek() {
                                if ch == '\n' { break; }
                                self.advance();
                            }
                            self.next_token()
                        } else if self.peek() == Some(&'*') {
                            self.advance();
                            while let Some(&ch) = self.peek() {
                                if ch == '*' {
                                    self.advance();
                                    if self.peek() == Some(&'/') {
                                        self.advance(); 
                                        break; 
                                    }
                                } else {
                                    self.advance();
                                }
                            }
                            self.next_token()
                        } else {
                            Token { kind: TokenType::Slash, line, column: col }
                        }
                    },
                    '(' => { self.advance(); Token { kind: TokenType::LParen, line, column: col } },
                    ')' => { self.advance(); Token { kind: TokenType::RParen, line, column: col } },
                    '{' => { self.advance(); Token { kind: TokenType::LBrace, line, column: col } },
                    '}' => { self.advance(); Token { kind: TokenType::RBrace, line, column: col } },
                    '[' => { self.advance(); Token { kind: TokenType::LBracket, line, column: col } },
                    ']' => { self.advance(); Token { kind: TokenType::RBracket, line, column: col } },

 
                    // Строки
                    '"' => self.lex_string(),
                    
                    // Числа
                    '0'..='9' => self.lex_number(),
                    
                    // Идентификаторы и ключевые слова
                    'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(),
                    
                    _ => {
                        self.advance();
                        Token { kind: TokenType::Unknown(c), line, column: col }
                    }
                }
            },
            None => Token { kind: TokenType::EOF, line, column: col },
        }
    }

    fn lex_string(&mut self) -> Token {
        self.advance(); // скипаем открывающую кавычку
        let start = self.pos;
        while let Some(&c) = self.peek() {
            if c == '"' {
                break;
            }
            self.advance();
        }
        let value: String = self.input[start..self.pos].iter().collect();
        self.advance(); // скипаем закрывающую кавычку
        Token { 
            kind: TokenType::StringLiteral(value), 
            line: self.line, // реальная линия немного сместится, но это ок для примера
            column: 0 
        }
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.pos;
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let value: String = self.input[start..self.pos].iter().collect();
        
        // Проверяем, не является ли это ключевым словом
        let kind = match value.as_str() {
            "package" => TokenType::Package,
            "using" => TokenType::Using,
            "func" => TokenType::Func,
            "public" => TokenType::Public,
            "static" => TokenType::Static,
            "local" => TokenType::Local,
            "struct" => TokenType::Struct,
            "interface" => TokenType::Interface,
            "enum" => TokenType::Enum,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "switch" => TokenType::Switch,
            "do" => TokenType::Do,
            "for" => TokenType::For,
            "return" => TokenType::Return,
            "mut" => TokenType::Mut,
            "new" => TokenType::New,
            "delete" => TokenType::Delete,
            "defer" => TokenType::Defer,
            "type" => TokenType::Type,
            "const" => TokenType::Const,
            "comptime" => TokenType::Comptime,
            "arena" => TokenType::Arena,
            "self" => TokenType::SelfKw,
            "int" => TokenType::Int,
            "short" => TokenType::Short,
            "long" => TokenType::Long,
            "float" => TokenType::Float,
            "double" => TokenType::Double,
            "bool" => TokenType::Bool,
            "char" => TokenType::Char,
            "byte" => TokenType::Byte,
            "string" => TokenType::String,
            "utype" => TokenType::Utype,
            "ptr" => TokenType::Ptr,
            "ref" => TokenType::Ref,
            "slice" => TokenType::Slice,
            "area" => TokenType::Area,
            "result" => TokenType::Result,
            "null" => TokenType::Null,
            "nullptr" => TokenType::Nullptr,
            "void" => TokenType::Void,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "default" => TokenType::Default,
            "continue" => TokenType::Continue,
            "break" => TokenType::Break,
            "record" => TokenType::Record,
            "case" => TokenType::Case,
            _ => TokenType::Identifier(value),
        };
        
        Token { kind, line: self.line, column: 0 }
    }
    
    fn lex_number(&mut self) -> Token {
        // Простая реализация для целых чисел
        let start = self.pos;
        while let Some(&c) = self.peek() {
            if c.is_digit(10) {
                self.advance();
            } else {
                break;
            }
        }
        let value: String = self.input[start..self.pos].iter().collect();
        let num = value.parse::<i64>().unwrap_or(0);
        Token { kind: TokenType::IntLiteral(num), line: self.line, column: 0 }
    }
}