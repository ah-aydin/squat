use crate::{
    chunk::Chunk,
    op_code::OpCode,
    compiler::{
        Compiler,
        CompileStatus
    }
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
    stack: Vec<f64>
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::with_capacity(INITIAL_STACK_SIZE)
        }
    }

    pub fn interpret_source(&mut self, source: String) -> InterpretResult {
        let mut chunk = Chunk::new("Base".to_owned());
        let mut compiler = Compiler::new(&source, &mut chunk);
        match compiler.compile() {
            CompileStatus::Success => self.interpret_chunk(&mut chunk),
            CompileStatus::Fail => InterpretResult::InterpretCompileError
        }
    }

    pub fn interpret_chunk(&mut self, chunk: &mut Chunk) -> InterpretResult {
        debug!("==== Interpret Chunk {} ====", chunk.get_name());
        chunk.reset();

        loop {
            for value in self.stack.iter() {
                debug!("[{}]", value);
            }

            chunk.disassemble_current_instruction();
            if let Some(instruction) = chunk.next() {
                match instruction {
                    OpCode::Constant => {
                        if let Some(OpCode::Index(index)) = chunk.next() {
                            let index = *index;
                            let constant: f64 = chunk.read_constant(index);
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
