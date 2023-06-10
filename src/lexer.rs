use std::{str::Chars, iter::Peekable};

use crate::token::{Token, TokenType};

#[derive(Debug)]
pub enum LexerError {
    UndefinedToken { line: u32, lexeme: String },
    IncompleteComment { line: u32 },
    IncompleteString { line: u32 },
    InternalError { msg: String, line: u32 }
}

pub struct Lexer<'a> {
    source: &'a str,
    start: usize,
    current_index: usize,
    source_iterator: Peekable<Chars<'a>>,
    line: u32
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a String) -> Lexer<'a> {
        Lexer {
            source,
            start: 0,
            current_index: 0,
            source_iterator: source.chars().peekable(),
            line: 1
        }
    }

    pub fn scan_token(&mut self) -> Result<Token, LexerError> {
        match self.comments_and_whitespaces() {
            Ok(result) => {
                if let Some(token) = result {
                    return Ok(token);
                }
            },
            Err(err) => {
                return Err(err);
            }
        };

        self.start = self.current_index;
        
        if let Some(c) = self.source_iterator.next() {
            self.current_index += 1;

            if let Some(token) = self.identifier(&c) {
                return Ok(token);
            }
            if let Some(token) = self.number(&c) {
                return Ok(token);
            }

            return match c {
                // Single-character
                '(' => Ok(self.make_token(TokenType::LeftParenthesis)),
                ')' => Ok(self.make_token(TokenType::RightParenthesis)),
                '{' => Ok(self.make_token(TokenType::LeftBrace)),
                '}' => Ok(self.make_token(TokenType::RightBrace)),
                '[' => Ok(self.make_token(TokenType::LeftBracket)),
                ']' => Ok(self.make_token(TokenType::RightBracket)),
                ',' => Ok(self.make_token(TokenType::Comma)),
                '.' => Ok(self.make_token(TokenType::Dot)),
                '-' => Ok(self.make_token(TokenType::Minus)),
                '+' => Ok(self.make_token(TokenType::Plus)),
                ';' => Ok(self.make_token(TokenType::Semicolon)),
                '/' => Ok(self.make_token(TokenType::Slash)),
                '*' => Ok(self.make_token(TokenType::Star)),
                ':' => Ok(self.make_token(TokenType::Colon)),
                '?' => Ok(self.make_token(TokenType::Question)),

                // One or two character tokens
                '!' => {
                    if let Some(c) = self.source_iterator.peek() {
                        if *c == '=' {
                            self.advance();
                            Ok(self.make_token(TokenType::BangEqual))
                        } else {
                            Ok(self.make_token(TokenType::Bang))
                        }
                    } else {
                        Err(
                            LexerError::InternalError {
                                msg: "Could not peek source_iterator".to_owned(),
                                line: self.line
                            }
                        )
                    }
                },
                '=' => {
                    if let Some(c) = self.source_iterator.peek() {
                        if *c == '=' {
                            self.advance();
                            Ok(self.make_token(TokenType::EqualEqual))
                        } else {
                            Ok(self.make_token(TokenType::Equal))
                        }
                    } else {
                        Err(
                            LexerError::InternalError {
                                msg: "Could not peek source_iterator".to_owned(),
                                line: self.line
                            }
                        )
                    }
                },
                '<' => {
                    if let Some(c) = self.source_iterator.peek() {
                        if *c == '=' {
                            self.advance();
                            Ok(self.make_token(TokenType::LessEqual))
                        } else {
                            Ok(self.make_token(TokenType::Less))
                        }
                    } else {
                        Err(
                            LexerError::InternalError {
                                msg: "Could not peek source_iterator".to_owned(),
                                line: self.line
                            }
                        )
                    }
                },
                '>' => {
                    if let Some(c) = self.source_iterator.peek() {
                        if *c == '=' {
                            self.advance();
                            Ok(self.make_token(TokenType::GreaterEqual))
                        } else {
                            Ok(self.make_token(TokenType::Greater))
                        }
                    } else {
                        Err(
                            LexerError::InternalError {
                                msg: "Could not peek source_iterator".to_owned(),
                                line: self.line
                            }
                        )
                    }
                },

                // Literals
                '"' => {
                    while let Some(c) = self.source_iterator.peek() {
                        match *c {
                            '\n' => self.line += 1,
                            '"' => break, 
                            _ => {}
                        };
                        self.advance();
                    }

                    if self.is_at_end() {
                        return Err(LexerError::IncompleteString { line: self.line });
                    }

                    self.advance();
                    Ok(self.make_token(TokenType::String))
                },
                _ => Err(LexerError::UndefinedToken { line: self.line, lexeme: (self.source[self.start..self.current_index]).to_owned(),  })
            };
        }

        Ok(self.make_token(TokenType::Eof))
    }

    /// It can return an optional token of type TokenType::Comment if it has encountered a comment
    /// It can also return a LexerError if it encounters an incomplete comment
    fn comments_and_whitespaces(&mut self) -> Result<Option<Token>, LexerError> {
        self.start = self.current_index;
        while let Some(c) = self.source_iterator.peek() {
            match c {
                ' ' => self.current_index += 1,
                '\r' => self.current_index += 1,
                '\t' => self.current_index += 1,
                '\n' => {
                    self.line += 1;
                    self.current_index += 1;
                },
                // Comments
                '/' => {
                    if self.peek_next("/") { // Single line
                        while let Some(c) = self.source_iterator.peek() {
                            if *c == '\n' {
                                break;
                            }
                            self.advance();
                        }

                        return Ok(Some(self.make_token(TokenType::Comment)));
                    } else if self.peek_next("*") { // Multi line
                        self.advance();
                        while let Some(c) = self.source_iterator.peek() {
                            if *c == '*' && self.peek_next("/") {
                                self.advance();
                                return Ok(Some(self.make_token(TokenType::Comment)));
                            }
                            self.advance();
                        }

                        return Err(LexerError::IncompleteComment { line: self.line })
                    }
                },
                _ => break
            }
            self.source_iterator.next();
        }
        Ok(None)
    }

    fn identifier(&mut self, c: &char) -> Option<Token> {
        if c.is_ascii_alphabetic() || *c == '_' {
            while let Some(c) = self.source_iterator.peek() {
                if c.is_ascii_alphabetic() || *c == '_' || c.is_numeric() {
                    self.advance();
                    continue;
                }
                break;
            }

            let lexeme = self.source.get(self.start..self.current_index).unwrap();
            return match lexeme {
                "and" => Some(self.make_token(TokenType::And)),
                "break" => Some(self.make_token(TokenType::Break)),
                "class" => Some(self.make_token(TokenType::Class)),
                "else" => Some(self.make_token(TokenType::Else)),
                "extends" => Some(self.make_token(TokenType::Extends)),
                "false" => Some(self.make_token(TokenType::False)),
                "for" => Some(self.make_token(TokenType::For)),
                "func" => Some(self.make_token(TokenType::Func)),
                "if" => Some(self.make_token(TokenType::If)),
                "nil" => Some(self.make_token(TokenType::Nil)),
                "or" => Some(self.make_token(TokenType::Or)),
                "print" => Some(self.make_token(TokenType::Print)),
                "return" => Some(self.make_token(TokenType::Return)),
                "static" => Some(self.make_token(TokenType::Static)),
                "super" => Some(self.make_token(TokenType::Super)),
                "this" => Some(self.make_token(TokenType::This)),
                "true" => Some(self.make_token(TokenType::True)),
                "var" => Some(self.make_token(TokenType::Var)),
                "while" => Some(self.make_token(TokenType::While)),
                _ => Some(self.make_token(TokenType::Identifier))
            }
        }
        None
    }

    fn number(&mut self, c: &char) -> Option<Token> {
        if c.is_numeric() {
            while let Some(d) = self.source_iterator.peek() {
                if d.is_numeric() {
                    self.advance();
                    continue;
                }
                break;
            }

            if let Some('.') = self.source_iterator.peek() {
                self.advance();
                while let Some(d) = self.source_iterator.peek() {
                    if d.is_numeric() {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }

            return Some(self.make_token(TokenType::Number));
        }
        None
    }

    fn advance(&mut self) {
        self.source_iterator.next();
        self.current_index += 1;
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            lexeme: (self.source[self.start..self.current_index]).to_owned(),
            line: self.line
        }
    }

    fn peek_next(&mut self, character: &str) -> bool {
        if let Some(substr) = self.source.get((self.current_index + 1)..(self.current_index + 2)) {
            if substr == character {
                self.advance();
                return true;
            }
        } else {
            return false;
        }
        false
    }

    fn is_at_end(&self) -> bool {
        self.current_index >= self.source.len()
    }
}
