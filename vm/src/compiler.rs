use std::collections::HashMap;

use crate::chunk::Chunk;
use crate::lexer::{Lexer, LexerError};
use crate::op_code::OpCode;
use crate::token::{TokenType, Token};
use crate::value::{SquatValue, ValueArray};

#[cfg(debug_assertions)]
use log::debug;

const INITIAL_LOCALS_VECTOR_SIZE: usize = 256;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment,
    Ternary,
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
    Success(usize, usize), // main_start, global_variable_count
    Fail
}

pub enum ChunkMode {
    Main,
    Global
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

    main_chunk: &'a mut Chunk,
    global_var_decl_chunk: &'a mut Chunk,
    chunk_mode: ChunkMode,

    global_variable_indicies: HashMap<String, usize>,
    constants: &'a mut ValueArray,
    functions: HashMap<String, (usize, usize)>, // (key, value) => (func_name, (start_index, arity))
    called_function: Option<String>,

    locals: Vec<Local>,
    scope_depth: u32,

    had_error: bool,
    panic_mode: bool,

    main_start: usize,
    found_main: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(
        source: &'a String,
        main_chunk: &'a mut Chunk,
        global_var_decl_chunk: &'a mut Chunk,
        constants: &'a mut ValueArray
    ) -> Compiler<'a> {
        Compiler {
            lexer: Lexer::new(source),
            previous_token: None,
            current_token: None,
            
            main_chunk,
            global_var_decl_chunk,
            chunk_mode: ChunkMode::Main,

            global_variable_indicies: HashMap::new(),
            constants,
            functions: HashMap::new(),
            called_function: None,

            locals: Vec::with_capacity(INITIAL_LOCALS_VECTOR_SIZE),
            scope_depth: 0,

            had_error: false,
            panic_mode: false,

            main_start: 0,
            found_main: false,
        }
    }

    pub fn compile(&mut self) -> CompileStatus {
        self.advance();

        while !self.check_current(TokenType::Eof) {
            self.declaration_global();
        }

        let mut compile_status = CompileStatus::Success(self.main_start, self.global_variable_indicies.len());

        if !self.found_main {
            compile_status = CompileStatus::Fail;
            println!("[COMPILE ERROR] Function 'main' was not defined!");
        }
        if self.had_error {
            compile_status = CompileStatus::Fail;
        }

        #[cfg(debug_assertions)]
        debug!("Global variable indicies {:?}", self.global_variable_indicies);
        #[cfg(debug_assertions)]
        debug!("Functions {:?}", self.functions);
        #[cfg(debug_assertions)]
        debug!("Constants {:?}", self.constants);

        compile_status
    }

    //////////////////////////////////////////////////////////////////////////
    /// Statement rules
    //////////////////////////////////////////////////////////////////////////
    
    fn declaration_global(&mut self) {
        if self.check_current(TokenType::Semicolon) {
            self.compile_warning("Unnecessary ';'");
        } else if self.check_current(TokenType::Func) {
            self.function_declaration();
        } else if self.check_current(TokenType::Var) {
            self.var_declaration(true);
        } else if self.check_current(TokenType::Return) {
            self.compile_error("Cannot return from outside a function.");
        } else {
            self.compile_error("Statements are not allowed outside of function blocks.");
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn declaration_function(&mut self) {
        if self.check_current(TokenType::Semicolon) {
            self.compile_warning("Unnecessary ';'");
        } else if self.check_current(TokenType::Func) {
            self.compile_error("You cannot define a function inside another function");
        } else if self.check_current(TokenType::Var) {
            self.var_declaration(false);
        } else if self.check_current(TokenType::Return) {
            self.return_statement();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn function_declaration(&mut self) {
        self.consume_current(TokenType::Identifier, "Expected an identifier after 'func'");
        let func_name = self.previous_token.as_ref().unwrap().lexeme.clone();

        self.consume_current(TokenType::LeftParenthesis, "Expect '(' after function name.");
        if func_name == "main" {
            if self.found_main {
                self.compile_error("Cannot have more then 1 main function");
            }
            self.begin_scope();

            self.found_main = true;
            self.consume_current(TokenType::RightParenthesis, "Expect closing ')'. Function 'main' does not take arguments."); 
            self.consume_current(TokenType::LeftBrace, "Expected '{' to define function body");

            self.write_op_code(OpCode::Start);
            self.main_start = self.main_chunk.get_size();

            self.block();
            self.write_op_code(OpCode::Stop);

            self.end_scope();
        } else {
            if self.functions.contains_key(&func_name) { // TODO consider adding function
                                                         // overloading
                self.compile_error(&format!("Function '{}' is already defined.", func_name));
            }
            self.begin_scope();

            let mut arity = 0;
            if !self.check_current(TokenType::RightParenthesis) {
                arity += 1;
                let constant = self.parse_variable("Expect parameter name").unwrap();
                self.define_variable(constant);

                while self.check_current(TokenType::Comma) {
                    arity += 1;
                    if arity > 255 {
                        self.compile_error("Can't have more then 255 parameters on a function");
                    }
                    let constant = self.parse_variable("Expect parameter name").unwrap();
                    self.define_variable(constant);
                }
                self.consume_current(TokenType::RightParenthesis, "Expect closing ')'.");
            }

            self.consume_current(TokenType::LeftBrace, "Expected '{' to define function body");

            self.write_op_code(OpCode::Start);
            self.functions.insert(func_name, (self.main_chunk.get_size() - 1, arity));
            
            self.block();
            self.end_scope();
            self.write_op_code(OpCode::Nil);
            self.write_op_code(OpCode::Return);
        }
    }

    fn var_declaration(&mut self, global: bool) {
        let index = match self.parse_variable("Expect variable name") {
            Ok(value) => value,
            Err(()) => {
                return;
            }
        };


        if global {
            self.chunk_mode = ChunkMode::Global;
        }

        if self.check_current(TokenType::Equal) {
            self.expression();
        } else {
            self.write_op_code(OpCode::Nil);
        }

        self.consume_current(TokenType::Semicolon, "Expect ';' after variable declaration.");

        self.define_variable(index);

        if global {
            self.chunk_mode = ChunkMode::Main;
        }
    }

    fn parse_variable(&mut self, error_msg: &str) -> Result<usize, ()> {
        self.consume_current(TokenType::Identifier, error_msg);

        if self.scope_depth > 0 {
            let name = self.previous_token.as_ref().unwrap().lexeme.clone();
            
            for i in (0..self.locals.len()).rev() {
                if let Some(depth) = self.locals[i].depth {
                    if depth < self.scope_depth {
                        break;
                    }
                } else {
                    self.compile_error("Can't read local variable in its own initializer.");
                }

                if self.locals[i].name == name {
                    self.compile_error(
                        &format!(
                            "Variable with name '{}' allready exists in this scope (depth: {})",
                            name,
                            &self.scope_depth
                        )
                    );
                    return Ok(0);
                }
            }
            let local = Local { name, depth: None };
            self.locals.push(local);
            return Ok(0);
        }

        let var_name = self.previous_token.as_ref().unwrap().lexeme.clone();
        if self.global_variable_indicies.get(&var_name).is_some() {
            self.compile_error(&format!("Variable {} is allready defined", var_name));
            return Err(());
        }

        let index = self.global_variable_indicies.len();
        self.global_variable_indicies.insert(var_name, self.global_variable_indicies.len());
        Ok(index)
    }

    fn define_variable(&mut self, index: usize) {
        if self.scope_depth > 0 {
            self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
            return;
        }

        self.write_op_code(OpCode::DefineGlobal(index));
    }

    fn return_statement(&mut self) {
        if self.check_current(TokenType::Semicolon) {
            self.write_op_code(OpCode::Nil);
            self.write_op_code(OpCode::Return);
            return;
        }
        self.expression();
        self.consume_current(TokenType::Semicolon, "Expected ';' after return value");
        self.write_op_code(OpCode::Return);
    }

    fn statement(&mut self) {
        if self.check_current(TokenType::Print) {
            self.print_statement()
        } else if self.check_current(TokenType::If) {
            self.if_statement();
        } else if self.check_current(TokenType::While) {
            self.while_statement();
        } else if self.check_current(TokenType::For) {
            self.for_statement();
        } else if self.check_current(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume_current(TokenType::Semicolon, "Expect ';' after value.");
        self.write_op_code(OpCode::Print);
    }

    fn if_statement(&mut self) {
        self.consume_current(TokenType::LeftParenthesis, "Expected '(' after 'if'");
        self.expression();
        self.consume_current(TokenType::RightParenthesis, "Expected closing ')'");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse(usize::MAX));
        self.write_op_code(OpCode::Pop);
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump(usize::MAX));
        self.patch_jump(then_jump);
        self.write_op_code(OpCode::Pop);

        if self.check_current(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let loop_start = self.main_chunk.get_size();
        self.consume_current(TokenType::LeftParenthesis, "Expected '(' after 'while'");
        self.expression();
        self.consume_current(TokenType::RightParenthesis, "Expected closing ')'");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse(usize::MAX));
        self.write_op_code(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.write_op_code(OpCode::Pop);
    }
    
    fn for_statement(&mut self) {
        self.begin_scope();

        self.consume_current(TokenType::LeftParenthesis, "Expected '(' after 'for'");
        if self.check_current(TokenType::Var) {
            self.var_declaration(false);
        } else if !self.check_current(TokenType::Semicolon) {
            self.expression_statement();
        }

        let mut loop_start = self.main_chunk.get_size();
        let mut exit_jump: Option<usize> = None;
        if !self.check_current(TokenType::Semicolon) {
            self.expression();
            self.consume_current(TokenType::Semicolon, "Expected ';' after loop condition");

            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse(usize::MAX)));
            self.write_op_code(OpCode::Pop);
        }

        if !self.check_current(TokenType::RightParenthesis) {
            let body_jump = self.emit_jump(OpCode::Jump(usize::MAX));
            let increment_start = self.main_chunk.get_size();
            self.expression();
            self.write_op_code(OpCode::Pop);
            self.consume_current(TokenType::RightParenthesis, "Expect closing ')'");
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.write_op_code(OpCode::Pop);
        }

        self.end_scope();
    }

    fn block(&mut self) {
        while !self.check_current(TokenType::RightBrace) {
            if self.check_current(TokenType::Eof) {
                self.compile_error("Expected closing '}' to end the block");
                break;
            }
            self.declaration_function();
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume_current(TokenType::Semicolon, "Expect ';' after expression");
        self.write_op_code(OpCode::Pop);
    }

    //////////////////////////////////////////////////////////////////////////
    /// Expression rules
    //////////////////////////////////////////////////////////////////////////
    
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        self.call_prefix(self.previous_token.as_ref().unwrap().token_type);

        while precedence <= self.get_precedence(self.current_token.as_ref().unwrap().token_type) {
            self.advance();

            if self.check_previous(TokenType::Question) {
                self.ternary();
                continue;
            }
            self.call_infix(self.previous_token.as_ref().unwrap().token_type);
        }
    }

    fn ternary(&mut self) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse(usize::MAX));
        self.write_op_code(OpCode::Pop);
        self.parse_precedence(Precedence::Ternary + 1);

        let end_jump = self.emit_jump(OpCode::Jump(usize::MAX));
        self.patch_jump(else_jump);
        self.write_op_code(OpCode::Pop);
        self.consume_current(TokenType::Colon, "Expect ':' after true ternary block");

        self.parse_precedence(Precedence::Ternary + 1);
        self.patch_jump(end_jump);
    }

    fn and(&mut self) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(usize::MAX));
        self.write_op_code(OpCode::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self) {
        let end_jump = self.emit_jump(OpCode::JumpIfTrue(usize::MAX));
        self.write_op_code(OpCode::Pop);
        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn binary(&mut self) {
        let token_type = self.previous_token.as_ref().unwrap().clone().token_type;

        let precedence = self.get_precedence(token_type);
        self.parse_precedence(precedence + 1);

        match token_type {
            TokenType::Plus             => self.write_op_code(OpCode::Add),
            TokenType::PlusPlus         => self.write_op_code(OpCode::Concat),
            TokenType::Minus            => self.write_op_code(OpCode::Subtract),
            TokenType::Star             => self.write_op_code(OpCode::Multiply),
            TokenType::Slash            => self.write_op_code(OpCode::Divide),
            TokenType::Percent          => self.write_op_code(OpCode::Mod),

            TokenType::BangEqual        => self.write_op_code(OpCode::NotEqual),
            TokenType::EqualEqual       => self.write_op_code(OpCode::Equal),
            TokenType::Greater          => self.write_op_code(OpCode::Greater),
            TokenType::GreaterEqual     => self.write_op_code(OpCode::GreaterEqual),
            TokenType::Less             => self.write_op_code(OpCode::Less),
            TokenType::LessEqual        => self.write_op_code(OpCode::LessEqual),

            _ => panic!("Unreachable line")
        }
    }

    fn call(&mut self) {
        if let Some(func_name) = &self.called_function {
            if let Some((jump_index, arity)) = self.functions.get(func_name) {
                let func_name = func_name.clone();
                let jump_index = *jump_index;
                let arity = *arity;

                let mut arg_count = 0;
                if !self.check_current(TokenType::RightParenthesis) {
                    arg_count += 1;
                    self.expression();

                    while self.check_current(TokenType::Comma) {
                        arg_count += 1;
                        self.expression();
                    }
                    self.consume_current(TokenType::RightParenthesis, "Expect closing ')'.");
                }

                if arg_count != arity {
                    self.compile_error(&format!("{} requires {} arguments, but {} were given", func_name, arity, arg_count));
                }
                self.write_op_code(OpCode::Call(jump_index, arity));
            }
        } else {
            panic!("Unreachable line");
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

        match token_type {
            TokenType::False => self.write_op_code(OpCode::False),
            TokenType::Nil => self.write_op_code(OpCode::Nil),
            TokenType::True => self.write_op_code(OpCode::True),
            _ => panic!("Unreachable line")
        }
    }

    fn number(&mut self) {
        let value: f64 = self.previous_token.as_ref().unwrap().lexeme.parse().unwrap();

        let index = self.constants.write(SquatValue::Number(value));
        self.write_op_code(OpCode::Constant(index));
    }

    fn string(&mut self) {
        let value: String = self.previous_token.as_ref().unwrap().lexeme.clone();

        let index = self.constants.write(SquatValue::String(value));
        self.write_op_code(OpCode::Constant(index));
    }

    fn unary(&mut self) {
        let token_type = self.previous_token.as_ref().unwrap().token_type;

        self.parse_precedence(Precedence::Unary);

        match token_type {
            TokenType::Bang => self.write_op_code(OpCode::Not),
            TokenType::Minus => self.write_op_code(OpCode::Negate),
            _ => panic!("Unreachable line")
        }
    }

    fn variable(&mut self) {
        let var_name = self.previous_token.as_ref().unwrap().lexeme.clone();

        let set_op_code: OpCode;
        let get_op_code: OpCode;

        if self.functions.contains_key(&var_name) {
            self.called_function = Some(var_name);
            return;
        }

        if let Some(index) = self.resolve_local(&var_name) {
            set_op_code = OpCode::SetLocal(index);
            get_op_code = OpCode::GetLocal(index);
        } else {
            if let Some(index) = self.global_variable_indicies.get(&var_name) {
                set_op_code = OpCode::SetGlobal(*index);
                get_op_code = OpCode::GetGlobal(*index);
            } else {
                self.compile_error(&format!("{} is not defined.", var_name));
                return;
            }
        }


        if self.check_current(TokenType::Equal) {
            self.expression();
            self.write_op_code(set_op_code);
        } else {
            self.write_op_code(get_op_code);
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
                            => self.compile_error_token(line, &format!("undefined token '{}'", lexeme)),
                        LexerError::IncompleteComment { line }
                            => self.compile_error_token(line, "incomplete comment"),
                        LexerError::IncompleteString { line }
                            => self.compile_error_token(line, "incomplete string"),
                        LexerError::InternalError { msg, line }
                            => self.compile_error_token(line, &msg)
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
            let lexeme = &self.previous_token.as_ref().unwrap().lexeme;
            self.compile_error(&format!("Error at '{}': {}", lexeme, message));
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

    fn check_previous(&self, expected_type: TokenType) -> bool {
        if let Some(token) = &self.previous_token {
            if token.token_type == expected_type {
                return true;
            }
        }
        return false;
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
                        break;
                    }
                _ => {}
            }
            self.advance();
        }
    }

    //////////////////////////////////////////////////////////////////////////
    /// Scope functions
    //////////////////////////////////////////////////////////////////////////

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // Remove the local variables from the stack
        while self.locals.len() > 0 && self.locals[self.locals.len() - 1].depth.unwrap() > self.scope_depth {
            self.write_op_code(OpCode::Pop);
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

    //////////////////////////////////////////////////////////////////////////
    /// Token Linkers
    //////////////////////////////////////////////////////////////////////////

    fn call_prefix(&mut self, token_type: TokenType) {
        match token_type {
            TokenType::LeftParenthesis                          => self.grouping(),
            TokenType::Bang | TokenType::Minus                  => self.unary(),
            TokenType::Number                                   => self.number(),
            TokenType::False | TokenType::Nil | TokenType::True => self.literal(),
            TokenType::String                                   => self.string(),
            TokenType::Identifier                               => self.variable(),
            TokenType::Eof                                      => return,
            _ => self.compile_error("This token is not siutable for an expression start")
        }
    }

    fn call_infix(&mut self, token_type: TokenType) {
        match token_type {
            TokenType::Minus | TokenType::Plus | TokenType::Slash | TokenType::Star |
                TokenType::PlusPlus | TokenType::Percent |
                TokenType::BangEqual | TokenType::EqualEqual |
                TokenType::Greater | TokenType::GreaterEqual |
                TokenType::Less | TokenType::LessEqual => self.binary(),
            TokenType::And => self.and(),
            TokenType::Or => self.or(),
            TokenType::LeftParenthesis => self.call(),
            _ => panic!("No infix is given for {:?}", token_type)
        }
    }

    fn get_precedence(&self, token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::Plus | TokenType::PlusPlus |
                TokenType::Minus | TokenType::Percent => Precedence::Term,
            TokenType::Star | TokenType::Slash => Precedence::Factor,
                TokenType::BangEqual | TokenType::EqualEqual => Precedence::Equality,
            TokenType::Greater | TokenType::GreaterEqual |
                TokenType::Less | TokenType::LessEqual => Precedence::Comparison,
            TokenType::And => Precedence::And,
            TokenType::Or => Precedence::Or,
            TokenType::Question => Precedence::Ternary,
            TokenType::LeftParenthesis => Precedence::Call,
            _ => Precedence::None
        }
    }

    //////////////////////////////////////////////////////////////////////////
    /// Jumps
    //////////////////////////////////////////////////////////////////////////
    
    fn emit_jump(&mut self, op_code: OpCode) -> usize {
        self.write_op_code(op_code);
        self.main_chunk.get_size() - 1
    }

    fn patch_jump(&mut self, op_location: usize) {
        let jump = self.main_chunk.get_size() - op_location - 1;
        self.main_chunk.set_jump_at(op_location, jump);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.write_op_code(OpCode::Loop(loop_start));
    }

    //////////////////////////////////////////////////////////////////////////
    /// Write instruction
    //////////////////////////////////////////////////////////////////////////

    fn write_op_code(&mut self, op_code: OpCode) {
        let line = self.previous_token.as_ref().unwrap().line;
        match self.chunk_mode {
            ChunkMode::Main => self.main_chunk.write(op_code, line),
            ChunkMode::Global => self.global_var_decl_chunk.write(op_code, line)
        };
    }

    //////////////////////////////////////////////////////////////////////////
    /// Logging
    //////////////////////////////////////////////////////////////////////////

    fn compile_error(&mut self, message: &str) {
        let line = self.previous_token.as_ref().unwrap().line;
        println!("[COMPILE ERROR] (Line {}) {}", line, message);
        self.had_error = true;
        self.panic_mode = true;
    }

    fn compile_error_token(&mut self, line: u32, message: &str) {
        println!("[COMPILE ERROR] (Line {}) {}", line, message);
        self.had_error = true;
        self.panic_mode = true;
    }

    fn compile_warning(&mut self, message: &str) {
        let line = self.previous_token.as_ref().unwrap().line;
        println!("[COMPILE WARNING] (Line {}) {}", line, message);
    }
}
