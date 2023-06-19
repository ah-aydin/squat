use std::collections::HashMap;

use crate::chunk::Chunk;
use crate::lexer::{Lexer, LexerError};
use crate::op_code::OpCode;
use crate::token::{TokenType, Token};
use crate::value::SquatValue;

#[cfg(debug_assertions)]
use log::debug;

const INITIAL_LOCALS_VECTOR_SIZE: usize = 256;

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

struct Local {
    name: String,
    // If this value is missing, the variable is not initialized yet.
    depth: Option<u32>
}

pub struct Compiler<'a> {
    lexer: Lexer<'a>,
    previous_token: Option<Token>,
    current_token: Option<Token>,

    chunk: &'a mut Chunk,
    global_variable_indicies: &'a mut HashMap<String, usize>,

    locals: Vec<Local>,
    scope_depth: u32,

    had_error: bool,
    panic_mode: bool
}

impl<'a> Compiler<'a> {
    pub fn new(
        source: &'a String,
        chunk: &'a mut Chunk,
        global_variable_indicies: &'a mut HashMap<String, usize>
    ) -> Compiler<'a> {
        Compiler {
            lexer: Lexer::new(source),
            previous_token: None,
            current_token: None,
            
            chunk,
            global_variable_indicies,

            locals: Vec::with_capacity(INITIAL_LOCALS_VECTOR_SIZE),
            scope_depth: 0,

            had_error: false,
            panic_mode: false
        }
    }

    pub fn compile(&mut self) -> CompileStatus {
        self.advance();

        while !self.check_current(TokenType::Eof) {
            self.declaration();
        }

        self.chunk.write(OpCode::Return, self.current_token.as_ref().unwrap().line);

        if self.had_error {
            return CompileStatus::Fail;
        }

        #[cfg(debug_assertions)]
        debug!("Global variable indicies {:?}", self.global_variable_indicies);

        CompileStatus::Success
    }

    //////////////////////////////////////////////////////////////////////////
    /// Statement rules
    //////////////////////////////////////////////////////////////////////////
    
    fn declaration(&mut self) {
        if self.check_current(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let line = self.previous_token.as_ref().unwrap().line;
        let index = self.parse_variable("Expect variable name");

        if self.check_current(TokenType::Equal) {
            self.expression();
        } else {
            self.chunk.write(OpCode::Nil, line);
        }

        self.consume_current(TokenType::Semicolon, "Expect ';' after variable declaration.");

        self.define_variable(index);
    }

    fn parse_variable(&mut self, error_msg: &str) -> usize {
        self.consume_current(TokenType::Identifier, error_msg);

        // Local variable
        if self.scope_depth > 0 {
            let name = self.previous_token.as_ref().unwrap().lexeme.clone();
            
            for i in (0..self.locals.len()).rev() {
                if let Some(depth) = self.locals[i].depth {
                    if depth < self.scope_depth {
                        break;
                    }
                } else {
                    let line = self.previous_token.as_ref().unwrap().line;
                    self.compile_error(line, "Can't read local variable in its own initializer.");
                }

                if self.locals[i].name == name {
                    let line = self.previous_token.as_ref().unwrap().line;
                    self.compile_error(
                        line,
                        &format!(
                            "Variable with name '{}' allready exists in this scope (depth: {})",
                            name,
                            &self.scope_depth
                        )
                    );
                    return 0;
                }
            }
            let local = Local { name, depth: None };
            self.locals.push(local);
            return 0;
        }

        let var_name = self.previous_token.as_ref().unwrap().lexeme.clone();
        if let Some(index) = self.global_variable_indicies.get(&var_name) {
            return *index;
        }

        let index = self.global_variable_indicies.len();
        self.global_variable_indicies.insert(var_name, self.global_variable_indicies.len());
        index
    }

    fn define_variable(&mut self, index: usize) {
        // Local variable
        if self.scope_depth > 0 {
            self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
            return;
        }

        let line = self.previous_token.as_ref().unwrap().line;
        self.chunk.write(OpCode::DefineGlobal, line);
        self.chunk.write(OpCode::Index(index), line);
    }

    fn statement(&mut self) {
        if self.check_current(TokenType::Print) {
            self.print_statement()
        } else if self.check_current(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        let line = self.previous_token.as_ref().unwrap().line;

        self.expression();
        self.consume_current(TokenType::Semicolon, "Expect ';' after value.");
        self.chunk.write(OpCode::Print, line);
    }

    fn block(&mut self) {
        while !self.check_current(TokenType::RightBrace) && !self.check_current(TokenType::Eof) {
            self.declaration();
        }

        self.consume_previous(TokenType::RightBrace, "Expect closing '}' to end the block");
    }

    fn expression_statement(&mut self) {
        let line = self.current_token.as_ref().unwrap().line;

        self.expression();
        self.consume_current(TokenType::Semicolon, "Expect ';' after expression");
        self.chunk.write(OpCode::Pop, line);
    }

    //////////////////////////////////////////////////////////////////////////
    /// Expression rules
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
        self.consume_current(TokenType::RightParenthesis, "Expected closing ')'");
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

    fn variable(&mut self) {
        let line = self.previous_token.as_ref().unwrap().line;

        let arg: usize;
        let var_name = self.previous_token.as_ref().unwrap().lexeme.clone();

        let set_op_code: OpCode;
        let get_op_code: OpCode;

        if let Some(local_arg) = self.resolve_local(&var_name) {
            arg = local_arg;

            set_op_code = OpCode::SetLocal;
            get_op_code = OpCode::GetLocal;
        } else {
            if let Some(index) = self.global_variable_indicies.get(&var_name) {
                arg = *index;
            } else {
                let index = self.global_variable_indicies.len();
                self.global_variable_indicies.insert(var_name, self.global_variable_indicies.len());
                arg = index;
            }

            set_op_code = OpCode::SetGlobal;
            get_op_code = OpCode::GetGlobal;
        }


        if self.check_current(TokenType::Equal) {
            self.expression();
            self.chunk.write(set_op_code, line);
            self.chunk.write(OpCode::Index(arg), line);
        } else {
            self.chunk.write(get_op_code, line);
            self.chunk.write(OpCode::Index(arg), line);
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

    fn consume_current(&mut self, expected_type: TokenType, message: &str) {
        if let Some(token) = &self.current_token {
            if token.token_type == expected_type {
                self.advance();
                return;
            }
            let line = self.previous_token.as_ref().unwrap().line;
            let lexeme = &self.previous_token.as_ref().unwrap().lexeme;
            self.compile_error(line, &format!("Error at '{}': {}", lexeme, message));
            return;
        }
        panic!("Unreachable line");
    }
    
    fn consume_previous(&mut self, expected_type: TokenType, message: &str) {
        if let Some(token) = &self.previous_token {
            if token.token_type == expected_type {
                return;
            }
            let line = self.previous_token.as_ref().unwrap().line;
            let lexeme = &self.previous_token.as_ref().unwrap().lexeme;
            self.compile_error(line, &format!("Error at '{}': {}", lexeme, message));
            return;
        }
        panic!("Unreachable line");
    }

    fn check_current(&mut self, expected_type: TokenType) -> bool {
        if let Some(token) = &self.current_token {
            if token.token_type == expected_type {
                self.advance();
                return true;
            }
        }
        false
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while self.current_token.as_ref().unwrap().token_type != TokenType::Eof {
            if self.current_token.as_ref().unwrap().token_type == TokenType::Semicolon {
                self.advance();
                break;
            }
            match self.current_token.as_ref().unwrap().token_type {
                TokenType::Class | TokenType::Func | TokenType::Var | TokenType::For |
                    TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => {
                        self.advance();
                        break;
                    }
                _ => {}
            }
            self.advance();
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // Remove the local variables from the stack
        while self.locals.len() > 0 && self.locals[self.locals.len() - 1].depth.unwrap() > self.scope_depth {
            let line = self.previous_token.as_ref().unwrap().line;
            self.chunk.write(OpCode::Pop, line);
            self.locals.pop();
        }
    }

    fn resolve_local(&mut self, name: &str) -> Option<usize> {
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].name == name && self.locals[i].depth.is_some() {
                return Some(i);
            }
        }
        None
    }

    fn call_prefix(&mut self, token_type: TokenType) {
        match token_type {
            TokenType::LeftParenthesis => self.grouping(),
            TokenType::Bang | TokenType::Minus => self.unary(),
            TokenType::Number => self.number(),
            TokenType::False | TokenType::Nil | TokenType::True => self.literal(),
            TokenType::String => self.string(),
            TokenType::Identifier => self.variable(),
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
        println!("[COMPILE ERROR] (Line {}) {}", line, message);
        self.had_error = true;
        self.panic_mode = true;
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
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        assert_eq!(chunk.next(), Some(&OpCode::False));
    }

    #[test]
    fn op_true() {
        let mut chunk = Chunk::new("True".to_owned());
        let source = String::from("true");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        assert_eq!(chunk.next(), Some(&OpCode::True));
    }

    #[test]
    fn add() {
        let mut chunk = Chunk::new("Addition".to_owned());
        let source = String::from("1 + 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Add);
    }

    #[test]
    fn subract() {
        let mut chunk = Chunk::new("Subtraction".to_owned());
        let source = String::from("1 - 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Subtract);
    }

    #[test]
    fn multiply() {
        let mut chunk = Chunk::new("Multiply".to_owned());
        let source = String::from("1 * 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Multiply);
    }

    #[test]
    fn divide() {
        let mut chunk = Chunk::new("Divide".to_owned());
        let source = String::from("1 / 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Divide);
    }

    #[test]
    fn concat() {
        let mut chunk = Chunk::new("Concat".to_owned());
        let source = String::from("\"a\" ++ \"b\"");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Concat);
    }

    #[test]
    fn equal() {
        let mut chunk = Chunk::new("Equal".to_owned());
        let source = String::from("1 == 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Equal);
    }

    #[test]
    fn not_equal() {
        let mut chunk = Chunk::new("Not Equal".to_owned());
        let source = String::from("1 != 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::NotEqual);
    }

    #[test]
    fn greater() {
        let mut chunk = Chunk::new("Greater".to_owned());
        let source = String::from("1 > 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Greater);
    }

    #[test]
    fn greater_equal() {
        let mut chunk = Chunk::new("Greater Equal".to_owned());
        let source = String::from("1 >= 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::GreaterEqual);
    }

    #[test]
    fn less() {
        let mut chunk = Chunk::new("Less".to_owned());
        let source = String::from("1 < 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::Less);
    }

    #[test]
    fn less_equal() {
        let mut chunk = Chunk::new("Less Equal".to_owned());
        let source = String::from("1 <= 2");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        interpret_binary_op_code(&mut chunk, OpCode::LessEqual);
    }

    #[test]
    fn not() {
        let mut chunk = Chunk::new("Less Equal".to_owned());
        let source = String::from("!1");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        assert_eq!(chunk.next(), Some(&OpCode::Constant));
        assert_eq!(chunk.next(), Some(&OpCode::Index(0)));
        assert_eq!(chunk.next(), Some(&OpCode::Not));
    }

    #[test]
    fn negate() {
        let mut chunk = Chunk::new("Less Equal".to_owned());
        let source = String::from("-1");
        let mut global_variable_indicies: HashMap<String, usize> = HashMap::new();
        let mut compiler = Compiler::new(&source, &mut chunk, &mut global_variable_indicies);
        compiler.compile();
        assert_eq!(chunk.next(), Some(&OpCode::Constant));
        assert_eq!(chunk.next(), Some(&OpCode::Index(0)));
        assert_eq!(chunk.next(), Some(&OpCode::Negate));
    }

    // TODO add more specific tests when not in flight and your brain can work
}
