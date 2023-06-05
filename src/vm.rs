use crate::{chunk::Chunk, op_code::OpCode};

use log::debug;

const INITIAL_STACK_SIZE: usize = 256;

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError
}

pub struct VM {
    chunk: Chunk,
    stack: Vec<f64>
}

impl VM {
    pub fn new(chunk: Chunk) -> VM {
        VM {
            chunk,
            stack: Vec::with_capacity(INITIAL_STACK_SIZE)
        }
    }

    pub fn interpret(&mut self) -> InterpretResult {
        debug!("==== Interpret Chunk {} ====", self.chunk.get_name());
        self.chunk.reset();

        loop {
            for value in self.stack.iter() {
                debug!("[{}]", value);
            }

            self.chunk.disassemble_current_instruction();
            if let Some(instruction) = self.chunk.next() {
                match instruction {
                    OpCode::Constant => {
                        if let Some(OpCode::Index(index)) = self.chunk.next() {
                            let index = *index;
                            let constant: f64 = self.chunk.read_constant(index);
                            self.stack.push(constant);
                        } else {
                            panic!("Constant OpCode must be followed by Index");
                        }
                    },
                    OpCode::Add => self.binary_op(|left, right| left + right),
                    OpCode::Subtract => self.binary_op(|left, right| left - right),
                    OpCode::Multiply => self.binary_op(|left, right| left * right),
                    OpCode::Divide => self.binary_op(|left, right| left / right),
                    OpCode::Negate => {
                        if let Some(value) = self.stack.pop() {
                            self.stack.push(-value);
                        } else {
                            panic!("OpCode Negate requires a value in the stack");
                        }
                    },
                    OpCode::Return => {
                        if let Some(value) = self.stack.pop() {
                            println!("{}", value);
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
            self.stack.push(op(left.unwrap(), right.unwrap()));
        } else {
            panic!("Binary operations require 2 values in the stack");
        }
    }
}
