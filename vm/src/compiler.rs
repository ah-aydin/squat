mod variable;

use std::collections::HashMap;

use crate::chunk::Chunk;
use crate::lexer::{Lexer, LexerError};
use crate::object::{SquatObject, SquatFunction};
use crate::op_code::OpCode;
use crate::token::{TokenType, Token};
use crate::value::squat_type::{SquatType, SquatFunctionTypeData};
use crate::value::{
    squat_value::SquatValue,
    ValueArray
};
use variable::{CompilerGlobal, CompilerLocal};

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
    Success(usize), // global_variable_count
    Fail
}

#[derive(Clone, Copy)]
enum ScopeType {
    Global,
    Function
}

pub struct Compiler<'a> {
    lexer: Lexer<'a>,
    previous_token: Option<Token>,
    current_token: Option<Token>,

    main_chunk: &'a mut Chunk,

    globals: HashMap<String, CompilerGlobal>,
    natives: &'a Vec<SquatValue>,
    constants: &'a mut ValueArray,

    locals: Vec<CompilerLocal>,
    scope_depth: u32,
    scope_type: ScopeType,
    scope_stack_index: usize,

    last_func_data: SquatType,
    had_error: bool,
    panic_mode: bool,

    main_start: usize,
    found_main: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(
        source: &'a String,
        main_chunk: &'a mut Chunk,
        constants: &'a mut ValueArray,
        natives: &'a Vec<SquatValue>
    ) -> Compiler<'a> {
        Compiler {
            lexer: Lexer::new(source),
            previous_token: None,
            current_token: None,
            
            main_chunk,

            globals: HashMap::new(),
            natives,
            constants,

            locals: Vec::with_capacity(INITIAL_LOCALS_VECTOR_SIZE),
            scope_depth: 0,
            scope_type: ScopeType::Global,
            scope_stack_index: 0,

            last_func_data: SquatType::Nil,
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
        self.main_chunk.write(OpCode::JumpTo(self.main_start), 0);

        let mut compile_status = CompileStatus::Success(self.globals.len());

        if !self.found_main {
            compile_status = CompileStatus::Fail;
            println!("[COMPILE ERROR] Function 'main' was not defined!");
        }
        if self.had_error {
            compile_status = CompileStatus::Fail;
        }

        #[cfg(debug_assertions)]
        println!("Global variable indicies {:?}", self.globals);
        #[cfg(debug_assertions)]
        println!("Constants {:?}", self.constants);

        compile_status
    }

    //////////////////////////////////////////////////////////////////////////
    /// Statement rules
    //////////////////////////////////////////////////////////////////////////

    fn try_var_declaration(&mut self) -> bool {
        if self.check_current(TokenType::Var) {
            self.var_declaration(None);
            return true;
        } else if self.check_current(TokenType::BoolType) {
            self.var_declaration(Some(SquatType::Bool));
            return true;
        } else if self.check_current(TokenType::IntType) {
            self.var_declaration(Some(SquatType::Int));
            return true;
        } else if self.check_current(TokenType::FloatType) {
            self.var_declaration(Some(SquatType::Float));
            return true;
        } else if self.check_current(TokenType::StringType) {
            self.var_declaration(Some(SquatType::String));
            return true;
        }
        false
    }

    fn function_type_declratation(&mut self) {
        let mut function_data: SquatFunctionTypeData = Default::default();
        if !self.check_current(TokenType::RightParenthesis) {
            function_data.param_types.push(match self.get_variable_type() {
                Ok(value) => value,
                Err(()) => return,
            });

            while self.check_current(TokenType::Comma) {
                function_data.param_types.push(match self.get_variable_type() {
                    Ok(value) => value,
                    Err(()) => return,
                });
            }
        }
        self.consume_current(TokenType::RightParenthesis, "Expect closing ')'.");

        function_data.set_return_type(match self.check_variable_type() {
            Some(value) => value,
            None => SquatType::Nil,
        });

        let function_type = SquatType::Function(function_data);

        self.var_declaration(Some(function_type));
    }
    
    fn declaration_global(&mut self) {
        if self.check_current(TokenType::Semicolon) {
            self.compile_warning("Unnecessary ';'");
        } else if self.check_current(TokenType::Func) {
            if self.check_current(TokenType::LeftParenthesis) {
                self.function_type_declratation();
            } else {
                self.function_declaration();
            }
        } else if self.try_var_declaration() {
        } else if self.check_current(TokenType::Return) {
            self.compile_error("Cannot return from outside a function.");
        } else {
            self.compile_error("Statements are not allowed outside of function blocks.");
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn declaration_local(&mut self, expected_return_type: SquatType) {
        if self.check_current(TokenType::Semicolon) {
            self.compile_warning("Unnecessary ';'");
        } else if self.check_current(TokenType::Func) {
            if self.check_current(TokenType::LeftParenthesis) {
                self.function_type_declratation();
            } else {
                self.function_declaration();
            }
        } else if self.try_var_declaration() {
        } else if self.check_current(TokenType::Return) {
            self.return_statement(expected_return_type);
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn function_declaration(&mut self) {
        let (index, func_name) = match self.parse_variable("Expect function name") {
            Ok(value) => value,
            Err(()) => {
                return;
            }
        };

        self.consume_current(TokenType::LeftParenthesis, "Expect '(' after function name.");
        let is_main: bool;
        if func_name == "main" {
            if self.found_main {
                self.compile_error("Cannot have more then 1 main function");
            }
            self.found_main = true;
            is_main = true;
        } else {
            is_main = false;
        }
        let old_scope_type = self.scope_type;
        let old_scope_stack_index = self.scope_stack_index;
        self.scope_stack_index = self.locals.len();
        self.scope_type = ScopeType::Function;

        if !is_main {
            self.initialize_function(&func_name);
        }
        self.begin_scope();

        let jump = self.emit_jump(OpCode::Jump(usize::MAX));
        let mut param_types: Vec<SquatType> = Vec::with_capacity(255);
        if !is_main {
            if !self.check_current(TokenType::RightParenthesis) {
                param_types.push(match self.get_variable_type() {
                    Ok(value) => value,
                    Err(()) => return,
                });
                let (constant, var_name) = match self.parse_variable("Expect parameter name") {
                    Ok((constant, var_name)) => (constant, var_name),
                    Err(_) => return,
                };
                self.define_variable(constant, &var_name, param_types.last().unwrap().clone());

                while self.check_current(TokenType::Comma) {
                    param_types.push(match self.get_variable_type() {
                        Ok(value) => value,
                        Err(()) => return,
                    });
                    let (constant, var_name) = match self.parse_variable("Expect parameter name") {
                        Ok((constant, var_name)) => (constant, var_name),
                        Err(_) => return,
                    };
                    self.define_variable(constant, &var_name, param_types.last().unwrap().clone());
                }
                self.consume_current(TokenType::RightParenthesis, "Expect closing ')'.");
            }
        } else {
            self.consume_current(TokenType::RightParenthesis, "Expect closing ')'");
        }

        let return_type: SquatType;
        if !is_main {
            return_type = match self.check_variable_type() {
                Some(value) => value,
                None => SquatType::Nil,
            };
        } else {
            return_type = SquatType::Int
        }

        self.consume_current(TokenType::LeftBrace, "Expected '{' to define function body");

        self.write_op_code(OpCode::Start);
        if is_main {
            self.main_start = self.main_chunk.get_size();
        }
        let starting_index = self.main_chunk.get_size() - 1;

        self.block(return_type.clone());
        self.end_scope();
        if is_main {
            self.write_op_code(OpCode::Stop);
        } else {
            self.write_op_code(OpCode::Nil);
            self.write_op_code(OpCode::Return);
        }

        self.patch_jump(jump);
        if !is_main {
            let arity = param_types.len();
            self.patch_function(
                &func_name,
                SquatFunctionTypeData::new(
                    arity,
                    param_types,
                    return_type
                    )
                );

            let function_obj = SquatObject::Function(
                SquatFunction::new(&func_name, starting_index, arity)
            );
            let constant_index = self.constants.write(SquatValue::Object(function_obj));
            self.write_op_code(OpCode::Constant(constant_index));
            self.define_function(index);
        }

        self.scope_type = old_scope_type;
        self.scope_stack_index = old_scope_stack_index;
    }

    fn var_declaration(&mut self, squat_type: Option<SquatType>) {
        let (index, name) = match self.parse_variable("Expect variable name") {
            Ok(value) => value,
            Err(()) => {
                return;
            }
        };

        let var_type: SquatType;

        if self.check_current(TokenType::Equal) {
            var_type = self.expression_with_type(squat_type);
        } else {
            if squat_type.is_none() {
                self.compile_error(&format!("Cannot define variable using 'var' without giving it a value"));
                return;
            }
            let index = match squat_type.unwrap() {
                SquatType::Int => {
                    var_type = SquatType::Int;
                    self.constants.write(SquatValue::Int(0))
                },
                SquatType::Float => {
                    var_type = SquatType::Float;
                    self.constants.write(SquatValue::Float(0.))
                },
                SquatType::String => {
                    var_type = SquatType::String;
                    self.constants.write(SquatValue::String("".to_owned()))
                },
                SquatType::Bool => {
                    var_type = SquatType::Bool;
                    self.constants.write(SquatValue::Bool(false))
                },
                _ => unreachable!("var_declaration")
            };
            self.write_op_code(OpCode::Constant(index));
        }

        self.consume_current(TokenType::Semicolon, "Expect ';' after variable declaration.");

        self.define_variable(index, &name, var_type);
    }

    fn get_variable_type(&mut self) -> Result<SquatType, ()> {
        if self.check_current(TokenType::BoolType)
            || self.check_current(TokenType::IntType)
            || self.check_current(TokenType::FloatType)
            || self.check_current(TokenType::StringType) {
            return match self.previous_token.as_ref().unwrap().token_type {
                TokenType::BoolType=> Ok(SquatType::Bool),
                TokenType::IntType => Ok(SquatType::Int),
                TokenType::FloatType => Ok(SquatType::Float),
                TokenType::StringType => Ok(SquatType::String),
                _ => unreachable!()
            };
        }
        self.compile_error("Expected variable type for function parameter");
        Err(())
    }

    fn check_variable_type(&mut self) -> Option<SquatType> {
        if self.check_current(TokenType::BoolType)
            || self.check_current(TokenType::IntType)
            || self.check_current(TokenType::FloatType)
            || self.check_current(TokenType::StringType) {
            return match self.previous_token.as_ref().unwrap().token_type {
                TokenType::BoolType=> Some(SquatType::Bool),
                TokenType::IntType => Some(SquatType::Int),
                TokenType::FloatType => Some(SquatType::Float),
                TokenType::StringType => Some(SquatType::String),
                _ => unreachable!()
            };
        }
        None
    }

    /// Return value:
    /// ```rust
    /// // If local
    /// Ok(0, variable_name: String)
    /// // If global
    /// Ok(global_index: usize, variable_name: String)
    /// ```
    fn parse_variable(&mut self, error_msg: &str) -> Result<(usize, String), ()> {
        self.consume_current(TokenType::Identifier, error_msg);

        let name = self.previous_token.as_ref().unwrap().lexeme.clone();

        if let Some(_) = self.resolve_native(&name) {
            self.compile_error(&format!("'{}' is a native object", name));
            return Err(());
        }

        if self.scope_depth > 0 {
            for i in (0..self.locals.len()).rev() {
                if let Some(depth) = self.locals[i].depth {
                    if depth < self.scope_depth {
                        break;
                    }
                }

                if self.locals[i].name == name {
                    self.compile_error(
                        &format!(
                            "'{}' allready exists in this scope (depth: {})",
                            name,
                            &self.scope_depth
                        )
                    );
                    return Err(());
                }
            }
            let local = CompilerLocal::new(&name, None, None);
            let index = self.locals.len();
            self.locals.push(local);
            return Ok((index, name));
        }

        let var_name = self.previous_token.as_ref().unwrap().lexeme.clone();
        if self.globals.get(&var_name).is_some() {
            self.compile_error(&format!("{} is allready defined", var_name));
            return Err(());
        }

        let index = self.globals.len();
        let global = CompilerGlobal::new(index, false, None);
        self.globals.insert(var_name, global);
        Ok((index, name))
    }

    fn initialize_function(&mut self, name: &str) {
        if self.scope_depth > 0 {
            self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
            return;
        }
        self.globals.get_mut(name).unwrap().initialized = true;
    }

    fn patch_function(&mut self, name: &str, data: SquatFunctionTypeData) {
        if self.scope_depth > 0 {
            self.locals.last_mut().unwrap().set_type(SquatType::Function(data));
            return;
        }
        self.globals.get_mut(name).unwrap().set_type(SquatType::Function(data));
    }

    fn define_function(&mut self, index: usize) {
        if self.scope_depth > 0 {
            return;
        }
        self.write_op_code(OpCode::DefineGlobal(index));
    }

    fn define_variable(&mut self, index: usize, name: &str, squat_type: SquatType) {
        if self.scope_depth > 0 {
            self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
            self.locals.last_mut().unwrap().set_type(squat_type);
            return;
        }
        self.globals.get_mut(name).unwrap().initialized = true;
        self.globals.get_mut(name).unwrap().set_type(squat_type);
        self.write_op_code(OpCode::DefineGlobal(index));
    }

    fn return_statement(&mut self, expected_return_type: SquatType) {
        let expression_type = self.expression();
        if expected_return_type != expression_type {
            self.compile_error(
                &format!(
                    "Function has return type '{}' but '{}' was given",
                    expected_return_type,
                    expression_type
                )
            );
        }
        self.consume_current(TokenType::Semicolon, "Expected ';' after return value");
        self.write_op_code(OpCode::Return);
    }

    fn statement(&mut self) {
        if self.check_current(TokenType::If) {
            self.if_statement();
        } else if self.check_current(TokenType::While) {
            self.while_statement();
        } else if self.check_current(TokenType::For) {
            self.for_statement();
        } else if self.check_current(TokenType::LeftBrace) {
            self.begin_scope();
            self.block(SquatType::Nil);
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn if_statement(&mut self) {
        self.consume_current(TokenType::LeftParenthesis, "Expected '(' after 'if'");
        self.expression(); // This expression can have any type, no type check required
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
        self.expression(); // This expression can have any type, no type check required
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
        if self.try_var_declaration() {
        } else if !self.check_current(TokenType::Semicolon) {
            self.expression_statement();
        }

        let mut loop_start = self.main_chunk.get_size();
        let mut exit_jump: Option<usize> = None;
        if !self.check_current(TokenType::Semicolon) {
            self.expression(); // This expression can have any type, no type check required
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

    fn block(&mut self, expected_return_type: SquatType) {
        while !self.check_current(TokenType::RightBrace) {
            if self.check_current(TokenType::Eof) {
                self.compile_error("Expected closing '}' to end the block");
                break;
            }
            self.declaration_local(expected_return_type.clone());
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
    
    fn parse_precedence(&mut self, precedence: Precedence, expected_type: Option<SquatType>) -> SquatType {
        self.advance();
        let prefix_type = self.call_prefix(
            self.previous_token.as_ref().unwrap().token_type,
            expected_type.clone()
        );
        if !self.check_types(expected_type.clone(), &prefix_type) {
            return expected_type.unwrap();
        }

        while precedence <= self.get_precedence(
            self.current_token.as_ref().unwrap().token_type
        ) {
            self.advance();

            if self.check_previous(TokenType::Question) {
                return self.ternary(expected_type);
            }
            self.call_infix(
                self.previous_token.as_ref().unwrap().token_type,
                Some(prefix_type.clone())
            );
        }

        prefix_type
    }

    fn ternary(&mut self, expected_type: Option<SquatType>) -> SquatType {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse(usize::MAX));
        self.write_op_code(OpCode::Pop);
        let expression_type = self.parse_precedence(Precedence::Ternary + 1, expected_type.clone());

        let end_jump = self.emit_jump(OpCode::Jump(usize::MAX));
        self.patch_jump(else_jump);
        self.write_op_code(OpCode::Pop);
        self.consume_current(TokenType::Colon, "Expect ':' after true ternary block");

        self.parse_precedence(Precedence::Ternary + 1, Some(expression_type.clone()));
        self.patch_jump(end_jump);

        expression_type
    }

    fn and(&mut self) -> SquatType {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(usize::MAX));
        self.write_op_code(OpCode::Pop);
        self.parse_precedence(Precedence::And, None);
        self.patch_jump(end_jump);
        SquatType::Bool
    }

    fn or(&mut self) -> SquatType {
        let end_jump = self.emit_jump(OpCode::JumpIfTrue(usize::MAX));
        self.write_op_code(OpCode::Pop);
        self.parse_precedence(Precedence::Or, None);
        self.patch_jump(end_jump);
        SquatType::Bool
    }

    fn binary(&mut self, expected_type: Option<SquatType>) -> SquatType {
        let token_type = self.previous_token.as_ref().unwrap().clone().token_type;

        let precedence = self.get_precedence(token_type);
        let rhs_type = self.parse_precedence(precedence + 1, expected_type.clone());
        self.check_types(expected_type, &rhs_type);

        match token_type {
            TokenType::Plus             => self.write_op_code(OpCode::Add),
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

            _ => unreachable!()
        }

        rhs_type
    }

    fn call(&mut self) -> SquatType {
        let func_data = self.last_func_data.clone();
        match func_data {
            SquatType::Function(data) => {
                let mut arg_count = 0;
                if !self.check_current(TokenType::RightParenthesis) {
                    while !self.check_current(TokenType::RightParenthesis) {
                        let expression_type = self.expression();
                        self.check_types(Some(data.get_param_type(arg_count)), &expression_type);
                        arg_count += 1;
                        self.check_current(TokenType::Comma);
                    }
                }

                self.write_op_code(OpCode::Call(arg_count));
                data.get_return_type()
            },
            SquatType::NativeFunction => {
                // TODO this is temporary code for the native calls
                let mut arg_count = 0;
                if !self.check_current(TokenType::RightParenthesis) {
                    while !self.check_current(TokenType::RightParenthesis) {
                        self.expression();
                        arg_count += 1;
                        self.check_current(TokenType::Comma);
                    }
                }

                self.write_op_code(OpCode::Call(arg_count));
                SquatType::Nil
            }
            _ => unreachable!("call")
        }
    }

    fn expression_with_type(&mut self, expected_type: Option<SquatType>) -> SquatType {
        self.parse_precedence(Precedence::Assignment, expected_type)
    }

    fn expression(&mut self) -> SquatType {
        self.expression_with_type(None)
    }

    fn grouping(&mut self, expected_type: Option<SquatType>) -> SquatType {
        let t = self.expression_with_type(expected_type);
        self.consume_current(TokenType::RightParenthesis, "Expected closing ')'");
        t
    }

    fn literal(&mut self) -> SquatType {
        let token_type = self.previous_token.as_ref().unwrap().token_type;

        match token_type {
            TokenType::False => {
                self.write_op_code(OpCode::False);
                SquatType::Bool
            },
            TokenType::Nil => {
                self.write_op_code(OpCode::Nil);
                SquatType::Nil
            },
            TokenType::True => {
                self.write_op_code(OpCode::True);
                SquatType::Bool
            },
            _ => unreachable!()
        }
    }

    fn number(&mut self) -> SquatType {
        let lexeme = &self.previous_token.as_ref().unwrap().lexeme;
        let index;
        let number_type: SquatType;
        if lexeme.contains(".") {
            let value: f64 = lexeme.parse().unwrap();
            index = self.constants.write(SquatValue::Float(value));
            number_type = SquatType::Float;
        } else {
            let value: i64 = lexeme.parse().unwrap();
            index = self.constants.write(SquatValue::Int(value));
            number_type = SquatType::Int;
        }

        self.write_op_code(OpCode::Constant(index));
        number_type
    }

    fn string(&mut self) -> SquatType {
        let value: String = self.previous_token.as_ref().unwrap().lexeme.clone();

        let index = self.constants.write(SquatValue::String(value));
        self.write_op_code(OpCode::Constant(index));
        SquatType::String
    }

    fn unary(&mut self, expected_type: Option<SquatType>) -> SquatType {
        let token_type = self.previous_token.as_ref().unwrap().token_type;

        let expression_type = self.parse_precedence(Precedence::Unary, expected_type.clone());
        self.check_types(expected_type, &expression_type);

        match token_type {
            TokenType::Bang => {
                self.write_op_code(OpCode::Not);
                SquatType::Bool
            },
            TokenType::Minus => {
                self.write_op_code(OpCode::Negate);
                expression_type
            }
            _ => unreachable!()
        }

    }

    fn variable(&mut self) -> SquatType {
        let var_name = self.previous_token.as_ref().unwrap().lexeme.clone();

        let set_op_code: OpCode;
        let get_op_code: OpCode;
        let variable_type: SquatType;
        let mut func_data: Option<SquatType> = None;
        let is_func: bool;

        if let Some((index, t, b)) = self.resolve_local(&var_name) {
            set_op_code = OpCode::SetLocal(index - self.scope_stack_index);
            get_op_code = OpCode::GetLocal(index - self.scope_stack_index);
            is_func = b;
            if is_func {
                func_data = Some(t.clone());
                variable_type = match t {
                    SquatType::Function(data) => data.get_return_type(),
                    _ => unreachable!()
                };
            } else {
                variable_type = t;
            }
        } else if let Some((index, t, b)) = self.resolve_global(&var_name) {
            set_op_code = OpCode::SetGlobal(index);
            get_op_code = OpCode::GetGlobal(index);
            is_func = b;
            if is_func {
                func_data = Some(t.clone());
                variable_type = match t {
                    SquatType::Function(data) => data.get_return_type(),
                    _ => unreachable!()
                };
            } else {
                variable_type = t;
            }
        } else if let Some((index, t, b)) = self.resolve_native(&var_name) {
            set_op_code = OpCode::Nil; // Just to keep the compiler happy
            get_op_code = OpCode::GetNative(index);
            is_func = b;
            if is_func {
                func_data = Some(t.clone());
                variable_type = match t {
                    // TODO temporary code
                    SquatType::NativeFunction => SquatType::Nil,
                    _ => unreachable!()
                };
            } else {
                variable_type = t;
            }
        }
        else {
            self.compile_error(&format!("{} is not defined.", var_name));
            return SquatType::Nil;
        }

        if self.check_current(TokenType::Equal) {
            if is_func {
                self.compile_error(
                    &format!(
                        "Cannot change assignment of function '{}",
                        var_name
                    )
                );
                return  SquatType::Nil;
            }
            if let OpCode::GetNative(_) = get_op_code {
                self.compile_error(
                    &format!(
                        "Cannot change assignment of native object '{}'",
                        var_name
                    )
                );
                return SquatType::Nil;
            }
            self.expression_with_type(Some(variable_type.clone()));
            self.write_op_code(set_op_code);
        } else {
            self.write_op_code(get_op_code);
        }

        if is_func {
            self.last_func_data = func_data.unwrap();
        }

        variable_type
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
                            => self.compile_error_token(
                                line,
                                &format!("undefined token '{}'", lexeme)
                            ),
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
        unreachable!();
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
        // TODO this function needs more work to function properly
        self.panic_mode = false;
        while self.current_token.as_ref().unwrap().token_type != TokenType::Eof {
            match self.current_token.as_ref().unwrap().token_type {
                TokenType::RightBrace | TokenType::Semicolon => {
                    self.advance();
                    break;
                },
                _ => {}
            }
            self.advance();
        }
    }

    fn check_types(&mut self, expected_type: Option<SquatType>, type_to_check: &SquatType) -> bool {
        if let Some(expected_type) = expected_type {
            if *type_to_check != expected_type {
                self.compile_error(
                    &format!(
                        "Expected {} but found {}",
                        expected_type,
                        type_to_check
                    )
                );
                return false;
            }
        }
        true
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
        while self.locals.len() > 0 &&
            self.locals[self.locals.len() - 1].depth.unwrap_or(0) > self.scope_depth
        {
            self.write_op_code(OpCode::Pop);
            self.locals.pop();
        }
    }

    fn resolve_native(&mut self, name: &str) -> Option<(usize, SquatType, bool)> {
        if let Some(native) = self.natives.iter().position(|x| match x {
            SquatValue::Object(SquatObject::NativeFunction(func)) => func.name == name,
            _ => unreachable!()
        }) {
            return Some((native, SquatType::NativeFunction, true));
        }
        None
    }

    fn resolve_global(&mut self, name: &str) -> Option<(usize, SquatType, bool)> {
        if let Some(global) = self.globals.get(name) {
            if global.initialized {
                let variable_type: SquatType = global.get_type();
                let is_func: bool;
                if let SquatType::Function(_data) = global.get_type() {
                    is_func = true;
                } else {
                    is_func = false;
                }
                return Some((global.index, variable_type, is_func));
            }
        }
        None
    }

    fn resolve_local(&mut self, name: &str) -> Option<(usize, SquatType, bool)> {
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].name == name && self.locals[i].depth.is_some() {
                let variable_type: SquatType = self.locals[i].get_type();
                let is_func: bool;
                if let SquatType::Function(_data) = self.locals[i].get_type() {
                    is_func = true;
                } else {
                    is_func = false;
                }
                return Some((i, variable_type, is_func));
            }
        }
        None
    }

    //////////////////////////////////////////////////////////////////////////
    /// Token Linkers
    //////////////////////////////////////////////////////////////////////////

    fn call_prefix(&mut self, token_type: TokenType, expected_type: Option<SquatType>) -> SquatType {
        match token_type {
            TokenType::LeftParenthesis                          => self.grouping(expected_type),
            TokenType::Bang | TokenType::Minus                  => self.unary(expected_type),
            TokenType::Number                                   => self.number(),
            TokenType::False | TokenType::Nil | TokenType::True => self.literal(),
            TokenType::String                                   => self.string(),
            TokenType::Identifier                               => self.variable(),
            TokenType::Eof                                      => SquatType::Nil,
            _ => { 
                self.compile_error("Illegal expression");
                SquatType::Nil
            }
        }
    }

    fn call_infix(&mut self, token_type: TokenType, expected_type: Option<SquatType>) -> SquatType {
        match token_type {
            TokenType::Minus | TokenType::Plus | TokenType::Slash | TokenType::Star |
                TokenType::Percent |
                TokenType::BangEqual | TokenType::EqualEqual |
                TokenType::Greater | TokenType::GreaterEqual |
                TokenType::Less | TokenType::LessEqual => self.binary(expected_type),
            TokenType::And => self.and(),
            TokenType::Or => self.or(),
            TokenType::LeftParenthesis => self.call(),
            _ => panic!("No infix is given for {:?}", token_type)
        }
    }

    fn get_precedence(&self, token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::Plus |
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
        self.main_chunk.write(op_code, line);
        return;
    }

    //////////////////////////////////////////////////////////////////////////
    /// Logging
    //////////////////////////////////////////////////////////////////////////

    fn compile_error(&mut self, message: &str) {
        let line = self.previous_token.as_ref().unwrap().line;
        println!("[ERROR] (Line {}) {}", line, message);
        self.had_error = true;
        self.panic_mode = true;
    }

    fn compile_error_token(&mut self, line: u32, message: &str) {
        println!("[ERROR] (Line {}) {}", line, message);
        self.had_error = true;
        self.panic_mode = true;
    }

    fn compile_warning(&mut self, message: &str) {
        let line = self.previous_token.as_ref().unwrap().line;
        println!("[WARNING] (Line {}) {}", line, message);
    }
}
