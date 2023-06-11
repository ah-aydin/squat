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
            //CompileStatus::Success => self.interpret_chunk(&mut chunk),
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
                    OpCode::Add => self.binary_op(|left, right| left + right),
                    OpCode::Subtract => self.binary_op(|left, right| left - right),
                    OpCode::Multiply => self.binary_op(|left, right| left * right),
                    OpCode::Divide => self.binary_op(|left, right| left / right),
                    OpCode::Negate => {
                        if let Some(SquatValue::F64(value)) = self.stack.pop() {
                            self.stack.push(SquatValue::F64(-value));
                        } else {
                            report_error(
                                self.chunk.get_current_instruction_line(),
                                "Negate must be used on a numeric value"
                            );
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
                    report_error(
                        self.chunk.get_current_instruction_line(),
                        "Left operand is not a numeric value"
                    );
                }
            }
            else {
                report_error(
                    self.chunk.get_current_instruction_line(),
                    "Right operand is not a numeric value"
                );
            }
        } else {
            panic!("Binary operations require 2 values in the stack");
        }
    }
}

fn report_error(line: u32, message: &str) {
    println!("[ERROR] (Line {}) {}", line, message);
}
