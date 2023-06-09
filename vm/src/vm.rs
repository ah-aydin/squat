use crate::{
    chunk::Chunk,
    op_code::OpCode,
    compiler::{
        Compiler,
        CompileStatus
    },
    value::{
        squat_value::SquatValue,
        ValueArray
    },
    native,
    options::Options,
    object::{
        SquatObject,
        SquatNativeFunction
    }
};

const INITIAL_STACK_SIZE: usize = 256;
const INITIAL_CALL_STACK_SIZE: usize = 256;

#[derive(PartialEq)]
pub enum InterpretResult {
    InterpretOk(i64),
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
    natives: Vec<SquatValue>,
    constants: ValueArray,
    current_chunk: usize,
    chunks: Vec<Chunk>,
    had_error: bool
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::with_capacity(INITIAL_STACK_SIZE),
            call_stack: Vec::with_capacity(INITIAL_CALL_STACK_SIZE),
            globals: vec![None; 1],
            natives: Vec::with_capacity(255),
            constants: ValueArray::new("Constants"),
            current_chunk: 0,
            chunks: vec![Chunk::new("Main", true)],
            had_error: false
        }
    }

    pub fn interpret_source(&mut self, source: String, opts: &Options) -> InterpretResult {
        self.define_native_functions();
        let mut compiler = Compiler::new(
            &source,
            &mut self.chunks[0],
            &mut self.constants,
            &self.natives
        );
        let compile_status = compiler.compile();

        drop(compiler);
        if opts.log_byte_code {
            println!("---------------- INSTRUCTIONS ----------------");
            self.chunks.iter().for_each(|chunk| chunk.disassemble());
            println!("----------------------------------------------");
        }

        let interpret_result = match compile_status {
            CompileStatus::Success(global_count) => {
                self.globals = vec![None; global_count];
                self.call_stack.push(
                    CallFrame::new(
                        0,
                        self.chunks[0].get_main_start(),
                        "main".to_owned()
                    )
                );

                self.interpret_chunk(0, opts)
            },
            CompileStatus::Fail => InterpretResult::InterpretCompileError
        };

        self.chunks[self.current_chunk].clear_instructions();
        interpret_result
    }

    fn interpret_chunk(&mut self, starting_instruction: usize, opts: &Options) -> InterpretResult {
        self.chunks[self.current_chunk].current_instruction = starting_instruction;

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
                self.chunks[self.current_chunk].disassemble_current_instruction();
            }

            if self.had_error {
                return InterpretResult::InterpretRuntimeError;
            }

            if let Some(instruction) = self.chunks[self.current_chunk].next() {
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
                    // Just to compile for now
                    OpCode::Mod             => self.binary_op(|left, right| left - right),
                    OpCode::Equal           => self.binary_cmp(|left, right| left == right),
                    OpCode::NotEqual        => self.binary_cmp(|left, right| left != right),
                    OpCode::Greater         => self.binary_cmp(|left, right| left > right),
                    OpCode::GreaterEqual    => self.binary_cmp(|left, right| left >= right),
                    OpCode::Less            => self.binary_cmp(|left, right| left < right),
                    OpCode::LessEqual       => self.binary_cmp(|left, right| left <= right),

                    OpCode::Not => {
                        if let Some(value) = self.stack.pop() {
                            self.stack.push(SquatValue::Bool(!value.is_truthy()));
                        } else {
                            panic!("'!' cannot be used alone");
                        }
                    }
                    OpCode::Negate => {
                        match self.stack.pop() {
                            Some(SquatValue::Float(value)) => {
                                self.stack.push(SquatValue::Float(-value));
                            },
                            Some(SquatValue::Int(value)) => {
                                self.stack.push(SquatValue::Int(-value));
                            },
                            _ => unreachable!("Negate requires a number value")
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
                            self.runtime_error(
                                &format!("Variable with index {} is not defined", index)
                            );
                        }
                    },
                    OpCode::SetGlobal(index) => {
                        let index = *index;
                        if let Some(value) = self.stack.last() {
                            if let Some(Some(_value)) = self.globals.get(index) {
                                self.globals[index] = Some(value.clone());
                            } else {
                                self.runtime_error(
                                    &format!("You cannot set a global variable before defining it")
                                );
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

                    OpCode::GetNative(index) => {
                        self.stack.push(self.natives[*index].clone());
                    },

                    OpCode::JumpTo(instruction_number) => {
                        self.chunks[self.current_chunk].current_instruction = *instruction_number;
                    }
                    OpCode::JumpIfFalse(offset) => {
                        if let Some(value) = self.stack.last() {
                            if !value.is_truthy() {
                                self.chunks[self.current_chunk].current_instruction += *offset;
                            }
                        } else {
                            panic!("JumpIfFalse OpCode expect a value to be on the stack");
                        }
                    },
                    OpCode::Jump(offset) => {
                        self.chunks[self.current_chunk].current_instruction += *offset;
                    },
                    OpCode::JumpIfTrue(offset) => {
                        if let Some(value) = self.stack.last() {
                            if value.is_truthy() {
                                self.chunks[self.current_chunk].current_instruction += *offset;
                            }
                        } else {
                            panic!("JumpIfTrue OpCode expect a value to be on the stack");
                        }
                    },
                    OpCode::Loop(loop_start) => {
                        self.chunks[self.current_chunk].current_instruction = *loop_start;
                    },

                    OpCode::Call(arg_count) => {
                        let arg_count = *arg_count;
                        let func_data_location = self.stack.len() - 1 - arg_count;
                        // All this ugly code for the native stuff exists because of the
                        // borrow checker.
                        let native = match self.stack.get(func_data_location).unwrap() {
                            SquatValue::Object(SquatObject::Function(func_data)) => {
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
                                let return_address = self.chunks[self.current_chunk].current_instruction;
                                self.call_stack.push(
                                    CallFrame::new(
                                        self.stack.len() - arg_count,
                                        return_address,
                                        func_data.name.clone()
                                        )
                                    );
                                self.chunks[self.current_chunk].current_instruction = func_data.start_instruction_index;
                                continue;
                            },
                            SquatValue::Object(SquatObject::NativeFunction(func)) => func.clone(),
                            _ => panic!("Call OpCode expects a FunctionObject on the stack")
                        };

                        if let Some(arity) = native.arity {
                            if arg_count != arity {
                                self.runtime_error(
                                    &format!(
                                        "Function takes {} arguments but {} were given",
                                        arity,
                                        arg_count
                                        )
                                    );
                                return InterpretResult::InterpretRuntimeError;
                            }
                        }

                        let mut args = Vec::new();
                        for _i in 0..arg_count {
                            args.push(self.stack.pop().unwrap())
                        }
                        self.stack.pop().unwrap();
                        args.reverse();
                        match native.call(args) {
                            Ok(value) => self.stack.push(value),
                            Err(msg) => self.runtime_error(&msg)
                        };
                    },
                    OpCode::Return => {
                        let return_val = self.stack.pop().unwrap();
                        if let Some(call_frame) = self.call_stack.pop() {
                            while call_frame.stack_index < self.stack.len() {
                                self.stack.pop(); // Pop local variables
                            }
                            self.stack.pop(); // Pop SquatFunc
                            self.chunks[self.current_chunk].current_instruction = call_frame.return_address;
                            self.stack.push(return_val);
                        } else {
                            if let SquatValue::Int(i) = return_val {
                                return InterpretResult::InterpretOk(i);
                            }
                            return InterpretResult::InterpretOk(0);
                        }
                    },

                    OpCode::Start => {},
                    OpCode::Stop => {
                        return InterpretResult::InterpretOk(0);
                    }
                }
            } else {
                break;
            }
        }

        InterpretResult::InterpretOk(0)
    }

    fn binary_op<F>(&mut self, op: F)
    where F: FnOnce(SquatValue, SquatValue) -> SquatValue {
        let right = self.stack.pop();
        let left = self.stack.pop();

        if left.is_some() && right.is_some() {
            self.stack.push(op(left.unwrap(), right.unwrap()));
        } else {
            unreachable!("Binary operations require 2 values in the stack");
        }
    }

    fn binary_cmp<F>(&mut self, op: F)
    where F: FnOnce(SquatValue, SquatValue) -> bool {
        let right = self.stack.pop();
        let left = self.stack.pop();

        if left.is_some() && right.is_some() {
            self.stack.push(SquatValue::Bool(op(left.unwrap(), right.unwrap())));
        } else {
            unreachable!("Binary comparisons require 2 values in the stack");
        }
    }

    fn runtime_error(&mut self, message: &str) {
        println!("Error callstack:");
        for call_frame in self.call_stack.iter().rev() {
            println!(
                "\tfunction '{}' called at line {}",
                call_frame.func_name,
                self.chunks[self.current_chunk].get_instruction_line(call_frame.return_address)
            );
        }
        println!(
            "[ERROR] (Line {}) {}",
            self.chunks[self.current_chunk].get_current_instruction_line(),
            message
        );
        self.had_error = true;
    }

    fn define_native_functions(&mut self) {
        self.define_native_func("input", Some(0), native::io::input);
        self.define_native_func("print", None, native::io::print);
        self.define_native_func("println", None, native::io::println);

        self.define_native_func("cbrt", Some(1), native::number::cbrt);
        self.define_native_func("sqrt", Some(1), native::number::sqrt);
        self.define_native_func("pow", Some(2), native::number::pow);
        self.define_native_func("number", Some(1), native::number::number);

        self.define_native_func("time", Some(0), native::misc::time);
        self.define_native_func("type", Some(1), native::misc::get_type);
    }

    fn define_native_func(&mut self, name: &str, arity: Option<usize>, func: native::NativeFunc) {
        let native_func = SquatNativeFunction::new(name, arity, func);
        let native_object = SquatObject::NativeFunction(native_func);
        let native_value = SquatValue::Object(native_object);
        self.natives.push(native_value);
    }
}
