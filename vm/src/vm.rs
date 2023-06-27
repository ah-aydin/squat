use crate::{
    chunk::Chunk,
    op_code::OpCode,
    compiler::{
        Compiler,
        CompileStatus
    }, value::{SquatValue, ValueArray, SquatObject},
    options::Options
};

const INITIAL_STACK_SIZE: usize = 256;
const INITIAL_CALL_STACK_SIZE: usize = 256;

#[derive(PartialEq)]
pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError
}

struct CallFrame {
    stack_index: usize,
    return_address: usize,
    func_name: String
}

impl CallFrame {
    fn new(stack_index: usize, return_address: usize, func_name: String) -> CallFrame {
        CallFrame {
            stack_index,
            return_address,
            func_name
        }
    }
}

pub struct VM {
    stack: Vec<SquatValue>,
    call_stack: Vec<CallFrame>,
    globals: Vec<Option<SquatValue>>,
    constants: ValueArray,
    main_chunk: Chunk,
    had_error: bool
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::with_capacity(INITIAL_STACK_SIZE),
            call_stack: Vec::with_capacity(INITIAL_CALL_STACK_SIZE),
            globals: vec![None; 1],
            constants: ValueArray::new("Constants"),
            main_chunk: Chunk::new("Main"),
            had_error: false
        }
    }

    pub fn interpret_source(&mut self, source: String, opts: &Options) -> InterpretResult {
        let mut compiler = Compiler::new(
            &source,
            &mut self.main_chunk,
            &mut self.constants
        );
        let interpret_result = match compiler.compile() {
            CompileStatus::Success(global_count) => {
                drop(compiler);
                if opts.log_byte_code {
                    println!("---------------- INSTRUCTIONS ----------------");
                    self.main_chunk.disassemble();
                    println!("----------------------------------------------");
                }
                self.globals = vec![None; global_count];
                self.call_stack.push(CallFrame::new(0, self.main_chunk.get_main_start(), "main".to_owned()));

                self.interpret_chunk(0, opts)
            },
            CompileStatus::Fail => InterpretResult::InterpretCompileError
        };

        self.main_chunk.clear_instructions();
        interpret_result
    }

    fn interpret_chunk(&mut self, starting_instruction: usize, opts: &Options) -> InterpretResult {
        self.main_chunk.current_instruction = starting_instruction;

        loop {
            if opts.log_stack {
                println!("STACK");
                for value in self.stack.iter() {
                    println!("\t[{:?}]", value);
                }
            }
            if opts.log_globals {
                println!("GLOBALS:");
                for (index, value) in self.globals.iter().enumerate() {
                    if let Some(value) = value {
                        println!("\t({}: {:?})", index, value);
                    }
                }
            }

            if opts.log_insturctions {
                self.main_chunk.disassemble_current_instruction();
            }

            if self.had_error {
                return InterpretResult::InterpretRuntimeError;
            }

            if let Some(instruction) = self.main_chunk.next() {
                match instruction {
                    OpCode::Constant(index) => {
                        let index = *index;
                        let constant: &SquatValue = self.constants.get(index);
                        self.stack.push(constant.clone());
                    },

                    OpCode::False           => self.stack.push(SquatValue::Bool(false)),
                    OpCode::Nil             => self.stack.push(SquatValue::Nil),
                    OpCode::True            => self.stack.push(SquatValue::Bool(true)),

                    OpCode::Add             => self.binary_op(|left, right| left + right),
                    OpCode::Subtract        => self.binary_op(|left, right| left - right),
                    OpCode::Multiply        => self.binary_op(|left, right| left * right),
                    OpCode::Divide          => self.binary_op(|left, right| left / right),
                    OpCode::Mod             => self.binary_op(|left, right| left % right),
                    OpCode::Equal           => self.binary_cmp(|left, right| left == right),
                    OpCode::NotEqual        => self.binary_cmp(|left, right| left != right),
                    OpCode::Greater         => self.binary_cmp(|left, right| left > right),
                    OpCode::GreaterEqual    => self.binary_cmp(|left, right| left >= right),
                    OpCode::Less            => self.binary_cmp(|left, right| left < right),
                    OpCode::LessEqual       => self.binary_cmp(|left, right| left <= right),

                    OpCode::Concat => {
                        let right = self.stack.pop();
                        let left = self.stack.pop();

                        if left.is_some() && right.is_some() {
                            self.stack.push(SquatValue::String(left.unwrap().to_string() + &right.unwrap().to_string()));
                        } else {
                            panic!("Concat operation requires 2 values in the stack");
                        }
                    },


                    OpCode::Not => {
                        if let Some(value) = self.stack.pop() {
                            self.stack.push(SquatValue::Bool(!value.is_truthy()));
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

                    OpCode::DefineGlobal(index) => {
                        let index = *index;
                        if let Some(value) = self.stack.pop() {
                            self.globals[index] = Some(value);
                        } else {
                            panic!("DefineGlobal OpCode expects a value to be on the stack");
                        }
                    },
                    OpCode::GetGlobal(index) => {
                        let index = *index;
                        if let Some(Some(value)) = self.globals.get(index) {
                            self.stack.push(value.clone());
                        } else {
                            self.runtime_error(&format!("Variable with index {} is not defined", index));
                        }
                    },
                    OpCode::SetGlobal(index) => {
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
                    },

                    OpCode::GetLocal(index) => {
                        let index = index + self.call_stack.last().unwrap().stack_index;
                        self.stack.push(self.stack[index].clone());
                    },
                    OpCode::SetLocal(index) => {
                        if let Some(value) = self.stack.last() {
                            let index = index + self.call_stack.last().unwrap().stack_index;
                            self.stack[index] = value.clone();
                        } else {
                            panic!("SetLocal OpCode expects a value to be on the stack");
                        }
                    },

                    OpCode::JumpTo(instruction_number) => {
                        self.main_chunk.current_instruction = *instruction_number;
                    }
                    OpCode::JumpIfFalse(offset) => {
                        if let Some(value) = self.stack.last() {
                            if !value.is_truthy() {
                                self.main_chunk.current_instruction += *offset;
                            }
                        } else {
                            panic!("JumpIfFalse OpCode expect a value to be on the stack");
                        }
                    },
                    OpCode::Jump(offset) => {
                        self.main_chunk.current_instruction += *offset;
                    },
                    OpCode::JumpIfTrue(offset) => {
                        if let Some(value) = self.stack.last() {
                            if value.is_truthy() {
                                self.main_chunk.current_instruction += *offset;
                            }
                        } else {
                            panic!("JumpIfTrue OpCode expect a value to be on the stack");
                        }
                    },
                    OpCode::Loop(loop_start) => {
                        self.main_chunk.current_instruction = *loop_start;
                    },

                    OpCode::Call(arg_count) => {
                        let arg_count = *arg_count;
                        let func_data_location = self.stack.len() - 1 - arg_count;
                        if let Some(SquatValue::Object(SquatObject::Function(func_data))) = self.stack.get(func_data_location) {
                            if arg_count != func_data.arity {
                                self.runtime_error(
                                    &format!(
                                        "Function takes {} arguments but {} were given",
                                        func_data.arity,
                                        arg_count
                                    )
                                );
                                return InterpretResult::InterpretRuntimeError;
                            }
                            let return_address = self.main_chunk.current_instruction;
                            self.call_stack.push(CallFrame::new(self.stack.len() - arg_count, return_address, func_data.name.clone()));
                            self.main_chunk.current_instruction = func_data.start_instruction_index;
                        } else {
                            panic!("Call OpCode expects a SquatValue::Object(SquatObject::Function(SquatFunction)) value on the stack");
                        }
                    },
                    OpCode::Return => {
                        if let Some(call_frame) = self.call_stack.pop() {
                            let return_val = self.stack.pop().unwrap();
                            while call_frame.stack_index < self.stack.len() {
                                self.stack.pop(); // Pop local variables
                            }
                            self.stack.pop(); // Pop SquatFunc
                            self.main_chunk.current_instruction = call_frame.return_address;
                            self.stack.push(return_val);
                        } else {
                            panic!("Return OpCode must contain a CallFrame in call_stack");
                        }
                    },

                    OpCode::Start => {},
                    OpCode::Stop => {
                        return InterpretResult::InterpretOk;
                    }
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
        println!("Error callstack:");
        for call_frame in self.call_stack.iter().rev() {
            // let line;
            // if call_frame.func_name == "main" {
            //     line = self.main_chunk.get_main
            // } else {
            //     line = self.main_chunk.get_instruction_line(call_frame.return_address);
            // }
            println!("\tfunction '{}' called at line {}", call_frame.func_name, self.main_chunk.get_instruction_line(call_frame.return_address));
        }
        println!("[ERROR] (Line {}) {}", self.main_chunk.get_current_instruction_line(), message);
        self.had_error = true;
    }
}
