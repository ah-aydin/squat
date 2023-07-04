use std::{str::Chars, iter::Peekable};

use crate::token::{Token, TokenType};

#[derive(Debug, PartialEq)]
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
                ';' => Ok(self.make_token(TokenType::Semicolon)),
                '/' => Ok(self.make_token(TokenType::Slash)),
                '*' => Ok(self.make_token(TokenType::Star)),
                '%' => Ok(self.make_token(TokenType::Percent)),
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
                '+' => {
                    if let Some(c) = self.source_iterator.peek() {
                        if *c == '+' {
                            self.advance();
                            Ok(self.make_token(TokenType::PlusPlus))
                        } else {
                            Ok(self.make_token(TokenType::Plus))
                        }
                    } else {
                        Err(
                            LexerError::InternalError {
                                msg: "Could not peek source_iterator".to_owned(),
                                line: self.line
                            }
                        )
                    }
                }

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

                    // Omit surrounding quotes
                    self.start += 1;
                    let token = self.make_token(TokenType::String);
                    self.advance();
                    Ok(token)
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
                                self.line += 1;
                                self.current_index += 1;
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next("*") { // Multi line
                        self.advance(); // Skip '*'
                        let mut complete_comment = false;

                        while let Some(c) = self.source_iterator.peek() {
                            if *c == '\n' {
                                self.line += 1;
                            }
                            if *c == '*' && self.peek_next("/") {
                                self.current_index += 1;
                                complete_comment = true;
                                break;
                            }
                            self.advance();
                        }

                        if !complete_comment {
                            return Err(LexerError::IncompleteComment { line: self.line })
                        }
                    } else {
                        break; // Break here to let it be handled as a Slash token
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
                "and" =>        Some(self.make_token(TokenType::And)),
                "break" =>      Some(self.make_token(TokenType::Break)),
                "class" =>      Some(self.make_token(TokenType::Class)),
                "else" =>       Some(self.make_token(TokenType::Else)),
                "extends" =>    Some(self.make_token(TokenType::Extends)),
                "false" =>      Some(self.make_token(TokenType::False)),
                "for" =>        Some(self.make_token(TokenType::For)),
                "func" =>       Some(self.make_token(TokenType::Func)),
                "if" =>         Some(self.make_token(TokenType::If)),
                "nil" =>        Some(self.make_token(TokenType::Nil)),
                "or" =>         Some(self.make_token(TokenType::Or)),
                "return" =>     Some(self.make_token(TokenType::Return)),
                "static" =>     Some(self.make_token(TokenType::Static)),
                "super" =>      Some(self.make_token(TokenType::Super)),
                "this" =>       Some(self.make_token(TokenType::This)),
                "true" =>       Some(self.make_token(TokenType::True)),
                "var" =>        Some(self.make_token(TokenType::Var)),
                "while" =>      Some(self.make_token(TokenType::While)),
                
                "bool" =>       Some(self.make_token(TokenType::BoolType)),
                "float" =>      Some(self.make_token(TokenType::FloatType)),
                "int" =>        Some(self.make_token(TokenType::IntType)),
                "string" =>     Some(self.make_token(TokenType::StringType)),

                _ =>            Some(self.make_token(TokenType::Identifier))
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::{Token, TokenType};

    fn make_token_line_1(token_type: TokenType, lexeme: &str) -> Result<Token, LexerError> {
        make_token(token_type, lexeme, 1)
    }

    fn make_token_line_2(token_type: TokenType, lexeme: &str) -> Result<Token, LexerError> {
        make_token(token_type, lexeme, 2)
    }

    fn make_token_line_3(token_type: TokenType, lexeme: &str) -> Result<Token, LexerError> {
        make_token(token_type, lexeme, 3)
    }

    fn make_token(token_type: TokenType,lexeme: &str, line: u32) -> Result<Token, LexerError> {
        Ok(Token {
            token_type,
            lexeme: lexeme.to_owned(),
            line
        })
    }

    fn test_binary_operand(token_type: TokenType, lexeme: &str) {
        let code = String::from(format!("1 {} 2", lexeme));
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(token_type, lexeme));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Eof, ""));
    }

    fn test_unary_operand(token_type: TokenType, lexeme: &str) {
        let code = String::from(format!("{}1", lexeme));
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(token_type, lexeme));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Eof, ""));
    }

    #[test]
    fn single_character_tokens() {
        let code = String::from("( ) { } [ ] , . ; : ?");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::LeftParenthesis, "("));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::RightParenthesis, ")"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::LeftBrace, "{"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::RightBrace, "}"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::LeftBracket, "["));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::RightBracket, "]"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Comma, ","));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Dot, "."));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Colon, ":"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Question, "?"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Eof, ""));

        test_unary_operand(TokenType::Minus, "-");
        test_binary_operand(TokenType::Minus, "-");
        test_binary_operand(TokenType::Star, "*");
        test_binary_operand(TokenType::Slash, "/");
    }

    #[test]
    fn one_or_two_character_tokens() {
        test_unary_operand(TokenType::Bang, "!");
        test_binary_operand(TokenType::BangEqual, "!=");
        test_binary_operand(TokenType::Equal, "=");
        test_binary_operand(TokenType::EqualEqual, "==");
        test_binary_operand(TokenType::Greater, ">");
        test_binary_operand(TokenType::GreaterEqual, ">=");
        test_binary_operand(TokenType::Less, "<");
        test_binary_operand(TokenType::LessEqual, "<=");
        test_binary_operand(TokenType::Plus, "+");
        test_binary_operand(TokenType::PlusPlus, "++");
    }

    #[test]
    fn literals() {
        let code = String::from("variable1 variable2 5 kebab chef");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "variable1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "variable2"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "5"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "kebab"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "chef"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Eof, ""));
        
        let code = String::from("\"kebab\" \"chef\" makes food");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::String, "kebab"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::String, "chef"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "makes"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "food"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Eof, ""));
    }

    #[test]
    fn keywords() {
        let code = String::from("and break class else extends false for func if nil or return static super this true var while");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::And, "and"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Break, "break"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Class, "class"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Else, "else"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Extends, "extends"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::False, "false"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::For, "for"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Func, "func"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::If, "if"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Nil, "nil"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Or, "or"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Return, "return"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Static, "static"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Super, "super"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::This, "this"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::True, "true"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::While, "while"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Eof, ""));
    }

    #[test]
    fn single_line_comment() {
        let code = String::from("var number1 = 1 + 2;// This is a comment");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "number1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Plus, "+"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Semicolon, ";"));
    }

    #[test]
    fn multi_line() {
        let code = String::from("var number1 = 1 + 2;\n\tvar number2 = 4 / 2;");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "number1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Plus, "+"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Identifier, "number2"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Number, "4"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Slash, "/"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Eof, ""));
    }

    #[test]
    fn multi_line_with_single_line_comment() {
        let code = String::from("var number1 = 1 + 2;\n// This is a comment\nvar number2 = 4 / 2;");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "number1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Plus, "+"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), make_token_line_3(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_3(TokenType::Identifier, "number2"));
        assert_eq!(lexer.scan_token(), make_token_line_3(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_3(TokenType::Number, "4"));
        assert_eq!(lexer.scan_token(), make_token_line_3(TokenType::Slash, "/"));
        assert_eq!(lexer.scan_token(), make_token_line_3(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_3(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), make_token_line_3(TokenType::Eof, ""));

        let code = String::from("var number1 = 1 + 2;\t\t\t   // This is a comment\nvar number2 = 4 / 2;");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "number1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Plus, "+"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Identifier, "number2"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Number, "4"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Slash, "/"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), make_token_line_2(TokenType::Eof, ""));
    }

    #[test]
    fn mutli_line_with_mulitline_comment() {
        let code = String::from("var number1 = 1 + 2;/* This is a \n multi \n line \n comment \t\tcomment*/\n  \tvar number2 = 4 / 2;");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "number1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Plus, "+"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Var, "var", 5));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Identifier, "number2", 5));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Equal, "=", 5));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Number, "4", 5));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Slash, "/", 5));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Number, "2", 5));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Semicolon, ";", 5));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Eof, "", 5));
    }

    #[test]
    fn detects_incomplete_multi_line_comment() {
        let code = String::from("var number1 = 1 + 2;/* This is an incomplete \n multi line \n comment \t\tcomment*\nvar number2 = 4 / 2;");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "number1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "1"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Plus, "+"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Number, "2"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Semicolon, ";"));
        assert_eq!(lexer.scan_token(), Err(LexerError::IncompleteComment { line: 4 }));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Eof, "", 4));
    }

    #[test]
    fn detects_incomplete_string() {
        let code = String::from("var s = \"gigel;");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Var, "var"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "s"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Equal, "="));
        assert_eq!(lexer.scan_token(), Err(LexerError::IncompleteString { line: 1}));
    }

    #[test]
    fn function_body() {
        let code = String::from("func main() { }");
        let mut lexer = Lexer::new(&code);
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Func, "func"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::Identifier, "main"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::LeftParenthesis, "("));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::RightParenthesis, ")"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::LeftBrace, "{"));
        assert_eq!(lexer.scan_token(), make_token_line_1(TokenType::RightBrace, "}"));
        assert_eq!(lexer.scan_token(), make_token(TokenType::Eof, "", 1));
    }
}
