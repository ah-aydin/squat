use crate::{
    chunk::Chunk,
    op_code::OpCode,
    compiler::{
        Compiler,
        CompileStatus
    }, value::SquatValue
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
    chunk: Chunk
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::with_capacity(INITIAL_STACK_SIZE),
            chunk: Chunk::new("Base".to_owned())
        }
    }

    pub fn interpret_source(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(&source, &mut self.chunk);
        let interpret_result = match compiler.compile() {
            CompileStatus::Success => self.interpret_chunk(),
            CompileStatus::Fail => InterpretResult::InterpretCompileError
        };

        self.chunk.clear_instructions();
        interpret_result
    }

    pub fn interpret_chunk(&mut self) -> InterpretResult {
        debug!("==== Interpret Chunk {} ====", self.chunk.get_name());
        self.chunk.reset();

        loop {
            for value in self.stack.iter() {
                debug!("[{:?}]", value);
            }

            #[cfg(debug_assertions)]
            self.chunk.disassemble_current_instruction();

            if let Some(instruction) = self.chunk.next() {
                match instruction {
                    OpCode::Constant => {
                        if let Some(OpCode::Index(index)) = self.chunk.next() {
                            let index = *index;
                            let constant: &SquatValue = self.chunk.read_constant(index);
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

                    OpCode::Equal => self.binary_cmp(|left, right| left == right),
                    OpCode::NotEqual => self.binary_cmp(|left, right| left != right),
                    OpCode::Greater => self.binary_cmp(|left, right| left > right),
                    OpCode::GreaterEqual => self.binary_cmp(|left, right| left >= right),
                    OpCode::Less => self.binary_cmp(|left, right| left < right),
                    OpCode::LessEqual => self.binary_cmp(|left, right| left <= right),

                    OpCode::Not => {
                        if let Some(value) = self.stack.pop() {
                            self.stack.push(SquatValue::Bool(!is_falsey(&value)));
                        } else {
                            panic!("'!' cannot be used alone");
                        }
                    }
                    OpCode::Negate => {
                        if let Some(SquatValue::F64(value)) = self.stack.pop() {
                            self.stack.push(SquatValue::F64(-value));
                        } else {
                            self.runtime_error("Negate must be used on a numeric value");
                            return InterpretResult::InterpretRuntimeError;
                        }
                    },

                    OpCode::Return => {
                        if let Some(value) = self.stack.pop() {
                            println!("{:?}", value);
                        }
                        return InterpretResult::InterpretOk;
                    },
                    _ => panic!("Unsupported Opcode {:?}", instruction)
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
            if let SquatValue::F64(right) = right.unwrap() {
                if let SquatValue::F64(left) = left.unwrap() {
                    self.stack.push(SquatValue::F64(op(left, right)));
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

    fn runtime_error(&self, message: &str) {
        println!("[ERROR] (Line {}) {}", self.chunk.get_current_instruction_line(), message);
    }
}

fn is_falsey(value: &SquatValue) -> bool {
    match value {
        SquatValue::Bool(true) => true,
        SquatValue::Bool(false) | SquatValue::Nil => false,
        _ => true
    }
}
