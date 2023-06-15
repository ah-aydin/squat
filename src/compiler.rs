use crate::chunk::Chunk;
use crate::lexer::{Lexer, LexerError};
use crate::op_code::OpCode;
use crate::token::{TokenType, Token};
use crate::value::SquatValue;

use log::error;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary
}

impl std::ops::Add<u8> for Precedence {
    type Output = Precedence;

    fn add(self, rhs: u8) -> Self::Output {
        let value = self as u8 + rhs;
        if value <= Precedence::Primary as u8 {
            return unsafe { std::mem::transmute::<u8, Precedence>(value) };
        }
        Precedence::None
    }
}

pub enum CompileStatus {
    Success,
    Fail
}

pub struct Compiler<'a> {
    lexer: Lexer<'a>,
    chunk: &'a mut Chunk,
    previous_token: Option<Token>,
    current_token: Option<Token>,
    had_error: bool
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a String, chunk: &'a mut Chunk) -> Compiler<'a> {
        Compiler {
            lexer: Lexer::new(source),
            chunk,
            previous_token: None,
            current_token: None,
            had_error: false
        }
    }

    pub fn compile(&mut self) -> CompileStatus {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expected end of expression");
        self.chunk.write(OpCode::Return, self.current_token.as_ref().unwrap().line);

        if self.had_error {
            return CompileStatus::Fail;
        }
        CompileStatus::Success
    }

    //////////////////////////////////////////////////////////////////////////
    /// Grammer rules
    //////////////////////////////////////////////////////////////////////////
    
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        self.call_prefix(self.previous_token.as_ref().unwrap().token_type);

        while precedence <= self.get_precedence(self.current_token.as_ref().unwrap().token_type) {
            self.advance();
            self.call_infix(self.previous_token.as_ref().unwrap().token_type);
        }
    }

    fn binary(&mut self) {
        let operation_token = self.previous_token.as_ref().unwrap().clone();

        let precedence = self.get_precedence(operation_token.token_type);
        self.parse_precedence(precedence + 1);

        match operation_token.token_type {
            TokenType::Plus =>          self.chunk.write(OpCode::Add, operation_token.line),
            TokenType::PlusPlus =>      self.chunk.write(OpCode::Concat, operation_token.line),
            TokenType::Minus =>         self.chunk.write(OpCode::Subtract, operation_token.line),
            TokenType::Star =>          self.chunk.write(OpCode::Multiply, operation_token.line),
            TokenType::Slash =>         self.chunk.write(OpCode::Divide, operation_token.line),

            TokenType::BangEqual =>     self.chunk.write(OpCode::NotEqual, operation_token.line),
            TokenType::EqualEqual =>    self.chunk.write(OpCode::Equal, operation_token.line),
            TokenType::Greater =>       self.chunk.write(OpCode::Greater, operation_token.line),
            TokenType::GreaterEqual =>  self.chunk.write(OpCode::GreaterEqual, operation_token.line),
            TokenType::Less =>          self.chunk.write(OpCode::Less, operation_token.line),
            TokenType::LessEqual =>     self.chunk.write(OpCode::LessEqual, operation_token.line),

            _ => panic!("Unreachable line")
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expected closing ')'");
    }

    fn literal(&mut self) {
        let token_type = self.previous_token.as_ref().unwrap().token_type;
        let line = self.previous_token.as_ref().unwrap().line;

        match token_type {
            TokenType::False => self.chunk.write(OpCode::False, line),
            TokenType::Nil => self.chunk.write(OpCode::Nil, line),
            TokenType::True => self.chunk.write(OpCode::True, line),
            _ => panic!("Unreachable line")
        }
    }

    fn number(&mut self) {
        let value: f64 = self.previous_token.as_ref().unwrap().lexeme.parse().unwrap();
        let line = self.previous_token.as_ref().unwrap().line;

        let index = self.chunk.add_constant(SquatValue::Number(value));
        self.chunk.write(OpCode::Constant, line);
        self.chunk.write(OpCode::Index(index), line);
    }

    fn string(&mut self) {
        let value: String = self.previous_token.as_ref().unwrap().lexeme.clone();
        let line = self.previous_token.as_ref().unwrap().line;

        let index = self.chunk.add_constant(SquatValue::String(value));
        self.chunk.write(OpCode::Constant, line);
        self.chunk.write(OpCode::Index(index), line);
    }

    fn unary(&mut self) {
        let token_type = self.previous_token.as_ref().unwrap().token_type;
        let line = self.previous_token.as_ref().unwrap().line;

        self.parse_precedence(Precedence::Unary);

        match token_type {
            TokenType::Bang => self.chunk.write(OpCode::Not, line),
            TokenType::Minus => self.chunk.write(OpCode::Negate, line),
            _ => panic!("Unreachable line")
        }
    }

    //////////////////////////////////////////////////////////////////////////
    /// Helper functions
    //////////////////////////////////////////////////////////////////////////

    fn advance(&mut self) {
        if self.current_token.is_some() {
            self.previous_token = Some(self.current_token.clone().unwrap());
        }

        loop {
            match self.lexer.scan_token() {
                Ok(token) => {
                    self.current_token = Some(token);
                    break;
                }
                Err(err) => {
                    match err {
                        LexerError::UndefinedToken { line, lexeme }
                            => self.compile_error(line, &format!("undefined token '{}'", lexeme)),
                        LexerError::IncompleteComment { line }
                            => self.compile_error(line, "incomplete comment"),
                        LexerError::IncompleteString { line }
                            => self.compile_error(line, "incomplete string"),
                        LexerError::InternalError { msg, line }
                            => self.compile_error(line, &msg)
                    };
                }
            }
        }
    }

    fn consume(&mut self, expected_type: TokenType, message: &str) {
        if self.current_token.is_some() {
            if self.current_token.as_ref().unwrap().token_type == expected_type {
                self.advance();
                return;
            }
            self.error_at_token(message);
            return;
        }
        panic!("Unreachable line");
    }

    fn error_at_token(&mut self, message: &str) {
        let line = self.current_token.as_ref().unwrap().line;
        let lexeme = &self.current_token.as_ref().unwrap().lexeme;
        error!("[Line: {}] Error at '{}': {}", line, lexeme, message);
        self.had_error = true;
    }

    fn call_prefix(&mut self, token_type: TokenType) {
        match token_type {
            TokenType::LeftParenthesis => self.grouping(),
            TokenType::Bang | TokenType::Minus => self.unary(),
            TokenType::Number => self.number(),
            TokenType::False | TokenType::Nil | TokenType::True => self.literal(),
            TokenType::String => self.string(),
            TokenType::Eof => return,
            _ => panic!("No prefix is given for {:?}", token_type)
        }
    }

    fn call_infix(&mut self, token_type: TokenType) {
        match token_type {
            TokenType::Minus | TokenType::Plus | TokenType::Slash | TokenType::Star |
                TokenType::PlusPlus |
                TokenType::BangEqual | TokenType::EqualEqual |
                TokenType::Greater | TokenType::GreaterEqual |
                TokenType::Less | TokenType::LessEqual => self.binary(),
            _ => panic!("No infix is given for {:?}", token_type)
        }
    }

    fn get_precedence(&self, token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::Plus | TokenType::PlusPlus | TokenType::Minus => Precedence::Term,
                TokenType::Star | TokenType::Slash => Precedence::Factor,
                TokenType::BangEqual | TokenType::EqualEqual => Precedence::Equality,
            TokenType::Greater | TokenType::GreaterEqual |
                TokenType::Less | TokenType::LessEqual => Precedence::Comparison,
            _ => Precedence::None
        }
    }

    fn compile_error(&mut self, line: u32, message: &str) {
        println!("[ERROR] (Line {}) {}", line, message);
        self.had_error = true;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn interpret_binary_op_code(chunk: &mut Chunk, op_code: OpCode) {
        assert_eq!(chunk.next(), Some(&OpCode::Constant));        
        assert_eq!(chunk.next(), Some(&OpCode::Index(0)));
        assert_eq!(chunk.next(), Some(&OpCode::Constant));
        assert_eq!(chunk.next(), Some(&OpCode::Index(1)));
        assert_eq!(chunk.next(), Some(&op_code));
    }

    #[test]
    fn op_false() {
        let mut chunk = Chunk::new("True".to_owned());
        let source = String::from("false");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        assert_eq!(chunk.next(), Some(&OpCode::False));
    }

    #[test]
    fn op_true() {
        let mut chunk = Chunk::new("True".to_owned());
        let source = String::from("true");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        assert_eq!(chunk.next(), Some(&OpCode::));
    }

    #[test]
    fn add() {
        let mut chunk = Chunk::new("Addition".to_owned());
        let source = String::from("1 + 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Add);
    }

    #[test]
    fn subract() {
        let mut chunk = Chunk::new("Subtraction".to_owned());
        let source = String::from("1 - 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Subtract);
    }

    #[test]
    fn multiply() {
        let mut chunk = Chunk::new("Multiply".to_owned());
        let source = String::from("1 * 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Multiply);
    }

    #[test]
    fn divide() {
        let mut chunk = Chunk::new("Divide".to_owned());
        let source = String::from("1 / 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Divide);
    }

    #[test]
    fn concat() {
        let mut chunk = Chunk::new("Concat".to_owned());
        let source = String::from("\"a\" ++ \"a\"");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Concat);
    }

    #[test]
    fn equal() {
        let mut chunk = Chunk::new("Equal".to_owned());
        let source = String::from("1 == 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Equal);
    }

    #[test]
    fn not_equal() {
        let mut chunk = Chunk::new("Not Equal".to_owned());
        let source = String::from("1 != 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::NotEqual);
    }

    #[test]
    fn greater() {
        let mut chunk = Chunk::new("Greater".to_owned());
        let source = String::from("1 > 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Greater);
    }

    #[test]
    fn greater_equal() {
        let mut chunk = Chunk::new("Greater Equal".to_owned());
        let source = String::from("1 >= 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::GreaterEqual);
    }

    #[test]
    fn less() {
        let mut chunk = Chunk::new("Less".to_owned());
        let source = String::from("1 < 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Less);
    }

    #[test]
    fn less_equal() {
        let mut chunk = Chunk::new("Less Equal".to_owned());
        let source = String::from("1 <= 2");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::LessEqual);
    }

    #[test]
    fn not() {
        let mut chunk = Chunk::new("Less Equal".to_owned());
        let source = String::from("!1");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        assert_eq!(chunk.next(), Some(&OpCode::Constant));
        assert_eq!(chunk.next(), Some(&OpCode::Index(0)));
        assert_eq!(chunk.next(), Some(&OpCode::Not));
    }

    #[test]
    fn negate() {
        let mut chunk = Chunk::new("Less Equal".to_owned());
        let source = String::from("-1");
        let mut compiler = Compiler::new(&source, &mut chunk);
        compiler.compile();
        assert_eq!(chunk.next(), Some(&OpCode::Constant));
        assert_eq!(chunk.next(), Some(&OpCode::Index(0)));
        assert_eq!(chunk.next(), Some(&OpCode::Negate));
    }

    // TODO add more specific tests when not in flight and your brain can work
}
