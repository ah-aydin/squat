use std::collections::HashMap;

use crate::{
    chunk::Chunk,
    op_code::OpCode,
    compiler::{
        Compiler,
        CompileStatus
    }, value::{SquatValue, ValueArray}
};

use log::debug;

const INITIAL_STACK_SIZE: usize = 256;

#[derive(PartialEq)]
pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError
}

pub struct VM {
    stack: Vec<SquatValue>,
    globals: Vec<Option<SquatValue>>,
    constants: ValueArray,
    main_chunk: Chunk,
    global_var_decl_chunk: Chunk,
    had_error: bool
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::with_capacity(INITIAL_STACK_SIZE),
            globals: vec![None; INITIAL_STACK_SIZE],
            constants: ValueArray::new("Constants"),
            main_chunk: Chunk::new("Main"),
            global_var_decl_chunk: Chunk::new("Global Variable Decl"),
            had_error: false
        }
    }

    pub fn interpret_source(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(
            &source,
            &mut self.main_chunk,
            &mut self.global_var_decl_chunk,
            &mut self.constants
        );
        let interpret_result = match compiler.compile() {
            CompileStatus::Success(main_start) => {
                drop(compiler);
                #[cfg(feature = "log_instructions")]
                {
                    println!("---------------- INSTRUCTIONS ----------------");
                    self.global_var_decl_chunk.disassemble();
                    self.main_chunk.disassemble();
                    println!("----------------------------------------------");
                }
                let starting_instruction = self.main_chunk.get_size();
                while let Some(instruction) = self.global_var_decl_chunk.next() {
                    self.main_chunk.write(*instruction, self.global_var_decl_chunk.get_current_instruction_line());
                }
                self.main_chunk.write(OpCode::JumpTo(main_start), 0);
                self.interpret_chunk(starting_instruction)
            },
            CompileStatus::Fail => InterpretResult::InterpretCompileError
        };

        self.main_chunk.clear_instructions();
        interpret_result
    }

    fn interpret_chunk(&mut self, starting_instruction: usize) -> InterpretResult {
        debug!("==== Interpret Chunk {} ====", self.main_chunk.get_name());
        self.main_chunk.current_instruction = starting_instruction;

        loop {
            #[cfg(feature = "log_stack")]
            {
                debug!("STACK");
                for value in self.stack.iter() {
                    debug!("[{:?}]", value);
                }
            }
            #[cfg(feature = "log_globals")]
            {
                debug!("GLOBALS");
                for (index, value) in self.globals.iter().enumerate() {
                    if let Some(value) = value {
                        debug!("({}: {:?})", index, value);
                    }
                }
            }

            #[cfg(debug_assertions)]
            self.main_chunk.disassemble_current_instruction();

            if self.had_error {
                return InterpretResult::InterpretRuntimeError;
            }

            if let Some(instruction) = self.main_chunk.next() {
                match instruction {
                    OpCode::Constant => {
                        if let Some(OpCode::Index(index)) = self.main_chunk.next() {
                            let index = *index;
                            let constant: &SquatValue = self.constants.get(index);
                            self.stack.push(constant.clone()); // TODO figure out a way to get rid
                                                               // of clone here
                        } else {
                            panic!("Constant OpCode must be followed by Index");
                        }
                    },

                    OpCode::False=> self.stack.push(SquatValue::Bool(false)),
                    OpCode::Nil => self.stack.push(SquatValue::Nil),
                    OpCode::True => self.stack.push(SquatValue::Bool(true)),

                    OpCode::Add => self.binary_op(|left, right| left + right),
                    OpCode::Subtract => self.binary_op(|left, right| left - right),
                    OpCode::Multiply => self.binary_op(|left, right| left * right),
                    OpCode::Divide => self.binary_op(|left, right| left / right),

                    OpCode::Concat => {
                        let right = self.stack.pop();
                        let left = self.stack.pop();

                        if left.is_some() && right.is_some() {
                            if let SquatValue::String(right) = right.unwrap() {
                                if let SquatValue::String(left) = left.unwrap() {
                                    self.stack.push(SquatValue::String(left + &right));
                                } else {
                                    self.runtime_error("Left operand is not a string");
                                }
                            }
                            else {
                                self.runtime_error("Right operand is not a string");
                            }
                        } else {
                            panic!("Concat operation requires 2 values in the stack");
                        }
                    },

                    OpCode::Equal => self.binary_cmp(|left, right| left == right),
                    OpCode::NotEqual => self.binary_cmp(|left, right| left != right),
                    OpCode::Greater => self.binary_cmp(|left, right| left > right),
                    OpCode::GreaterEqual => self.binary_cmp(|left, right| left >= right),
                    OpCode::Less => self.binary_cmp(|left, right| left < right),
                    OpCode::LessEqual => self.binary_cmp(|left, right| left <= right),

                    OpCode::Not => {
                        if let Some(value) = self.stack.pop() {
                            self.stack.push(SquatValue::Bool(!is_truthy(&value)));
                        } else {
                            panic!("'!' cannot be used alone");
                        }
                    }
                    OpCode::Negate => {
                        if let Some(SquatValue::Number(value)) = self.stack.pop() {
                            self.stack.push(SquatValue::Number(-value));
                        } else {
                            self.runtime_error("Negate must be used on a numeric value");
                            return InterpretResult::InterpretRuntimeError;
                        }
                    },

                    OpCode::Print => {
                        if let Some(value) = self.stack.pop() {
                            println!("{}", value);
                        }
                    },
                    OpCode::Pop => {
                        self.stack.pop();
                    },

                    OpCode::DefineGlobal => {
                        if let Some(OpCode::Index(index)) = self.main_chunk.next() {
                            let index = *index;
                            if let Some(value) = self.stack.pop() {
                                self.globals.insert(index, Some(value));
                            } else {
                                panic!("DefineGlobal OpCode expects a value to be on the stack");
                            }
                        } else {
                            panic!("DefineGlobal OpCode must be followed by Index OpCode");
                        }
                    },
                    OpCode::GetGlobal => {
                        if let Some(OpCode::Index(index)) = self.main_chunk.next() {
                            let index = *index;
                            if let Some(Some(value)) = self.globals.get(index) {
                                self.stack.push(value.clone());
                            } else {
                                self.runtime_error(&format!("Variable with index {} is not defined", index));
                            }
                        } else {
                            panic!("GetGlobal OpCode must be followed by Index OpCode");
                        }
                    },
                    OpCode::SetGlobal => {
                        if let Some(OpCode::Index(index)) = self.main_chunk.next() {
                            let index = *index;
                            if let Some(value) = self.stack.last() {
                                if let Some(Some(_value)) = self.globals.get(index) {
                                    self.globals[index] = Some(value.clone());
                                } else {
                                    self.runtime_error(&format!("You cannot set a global variable before defining it"));
                                }
                            } else {
                                panic!("SetGlobal OpCode expects a value to be on the stack");
                            }
                        } else {
                            panic!("SetGlobal OpCode must be followed by Index OpCode");
                        }
                    },

                    OpCode::GetLocal => {
                        if let Some(OpCode::Index(index)) = self.main_chunk.next() {
                            self.stack.push(self.stack[*index].clone());
                        } else {
                            panic!("GetLocal OpCode must be followed by Index OpCode");
                        }
                    },
                    OpCode::SetLocal => {
                        if let Some(OpCode::Index(index)) = self.main_chunk.next() {
                            if let Some(value) = self.stack.last() {
                                self.stack[*index] = value.clone();
                            } else {
                                panic!("SetLocal OpCode expects a value to be on the stack");
                            }
                        } else {
                            panic!("SetLocal OpCode must be followed by Index OpCode");
                        }
                    },

                    OpCode::JumpTo(instruction_number) => {
                        self.main_chunk.current_instruction = *instruction_number;
                    }
                    OpCode::JumpIfFalse => {
                        if let Some(OpCode::JumpOffset(offset)) = self.main_chunk.next() {
                            if let Some(value) = self.stack.last() {
                                if !is_truthy(value) {
                                    self.main_chunk.current_instruction += offset.clone();
                                }
                            } else {
                                panic!("JumpIfFalse OpCode expect a value to be on the stack");
                            }
                        } else {
                            panic!("JumpIfFalse OpCode must be followed by JumpOffset OpCode");
                        }
                    },
                    OpCode::Jump => {
                        if let Some(OpCode::JumpOffset(offset)) = self.main_chunk.next() {
                            self.main_chunk.current_instruction += offset.clone();
                        } else {
                            panic!("Jump OpCode must be followd by JumpOffset OpCode");
                        }
                    },
                    OpCode::JumpIfTrue => {
                        if let Some(OpCode::JumpOffset(offset)) = self.main_chunk.next() {
                            if let Some(value) = self.stack.last() {
                                if is_truthy(value) {
                                    self.main_chunk.current_instruction += offset.clone();
                                }
                            } else {
                                panic!("JumpIfTrue OpCode expect a value to be on the stack");
                            }
                        } else {
                            panic!("JumpIfTrue OpCode must be followed by JumpOffset OpCode");
                        }
                    },
                    OpCode::Loop => {
                        if let Some(OpCode::JumpOffset(offset)) = self.main_chunk.next() {
                            self.main_chunk.current_instruction -= offset.clone();
                        } else {
                            panic!("Loop OpCode must be followd by JumpOffset OpCode");
                        }
                    }

                    OpCode::Return => {
                    },

                    OpCode::Start => {},
                    OpCode::Stop => {
                        return InterpretResult::InterpretOk;
                    }

                    _ => panic!("Unsupported OpCode {:?}", instruction)
                }
            } else {
                break;
            }
        }

        InterpretResult::InterpretOk
    }

    fn binary_op<F>(&mut self, op: F)
    where F: FnOnce(f64, f64) -> f64 {
        let right = self.stack.pop();
        let left = self.stack.pop();

        if left.is_some() && right.is_some() {
            if let SquatValue::Number(right) = right.unwrap() {
                if let SquatValue::Number(left) = left.unwrap() {
                    self.stack.push(SquatValue::Number(op(left, right)));
                } else {
                    self.runtime_error("Left operand is not a numeric value");
                }
            }
            else {
                self.runtime_error("Right operand is not a numeric value");
            }
        } else {
            panic!("Binary operations require 2 values in the stack");
        }
    }

    fn binary_cmp<F>(&mut self, op: F)
    where F: FnOnce(SquatValue, SquatValue) -> bool {
        let right = self.stack.pop();
        let left = self.stack.pop();

        if left.is_some() && right.is_some() {
            self.stack.push(SquatValue::Bool(op(left.unwrap(), right.unwrap())));
        } else {
            panic!("Binary comparisons require 2 values in the stack");
        }
    }

    fn runtime_error(&mut self, message: &str) {
        println!("[ERROR] (Line {}) {}", self.main_chunk.get_current_instruction_line(), message);
        self.had_error = true;
    }
}

fn is_truthy(value: &SquatValue) -> bool {
    match value {
        SquatValue::Bool(true) => true,
        SquatValue::Bool(false) | SquatValue::Nil => false,
        _ => true
    }
}
