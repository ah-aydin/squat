use crate::chunk::Chunk;
use crate::lexer::Lexer;
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

    fn binary(&mut self) {
        let operation_token = self.previous_token.as_ref().unwrap().clone();

        let precedence = self.get_precedence(operation_token.token_type);
        self.parse_precedence(precedence + 1);

        match operation_token.token_type {
            TokenType::Plus => self.chunk.write(OpCode::Add, operation_token.line),
            TokenType::Minus => self.chunk.write(OpCode::Subtract, operation_token.line),
            TokenType::Star => self.chunk.write(OpCode::Multiply, operation_token.line),
            TokenType::Slash => self.chunk.write(OpCode::Divide, operation_token.line),
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

    fn number(&mut self) {
        let value: f64 = self.previous_token.as_ref().unwrap().lexeme.parse().unwrap();
        let line = self.previous_token.as_ref().unwrap().line;

        let index = self.chunk.add_constant(SquatValue::F64(value));
        self.chunk.write(OpCode::Constant, line);
        self.chunk.write(OpCode::Index(index), line);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        self.call_prefix(self.previous_token.as_ref().unwrap().token_type);

        while precedence <= self.get_precedence(self.current_token.as_ref().unwrap().token_type) {
            self.advance();
            self.call_infix(self.previous_token.as_ref().unwrap().token_type);
        }
    }

    fn unary(&mut self) {
        let token_type = self.previous_token.as_ref().unwrap().token_type;
        let line = self.previous_token.as_ref().unwrap().line;

        self.parse_precedence(Precedence::Unary);

        match token_type {
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
                    error!("{:?}", err);
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
            TokenType::Minus => self.unary(),
            TokenType::Number => self.number(),
            _ => panic!("No prefix is given for {:?}", token_type)
        }
    }

    fn call_infix(&mut self, token_type: TokenType) {
        match token_type {
            TokenType::Minus | TokenType::Plus | TokenType::Slash | TokenType::Star => self.binary(),
            _ => panic!("No prefix is given for {:?}", token_type)
        }
    }

    fn get_precedence(&self, token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::Plus | TokenType::Minus => Precedence::Term,
            TokenType::Star | TokenType::Slash => Precedence::Factor,
            _ => Precedence::None
        }
    }
}
