use crate::{
    chunk::Chunk,
    compiler::{variable::CompilerNative, CompileStatus, Compiler},
    native,
    object::{SquatInstance, SquatNativeFunction, SquatObject},
    op_code::OpCode,
    options::Options,
    value::{
        squat_type::{SquatFunctionTypeData, SquatType},
        squat_value::SquatValue,
        ValueArray,
    },
};

const INITIAL_STACK_SIZE: usize = 256;
const INITIAL_CALL_STACK_SIZE: usize = 256;

#[derive(PartialEq)]
pub enum InterpretResult {
    InterpretOk(i64),
    InterpretCompileError,
    InterpretRuntimeError,
}

struct CallFrame {
    stack_index: usize,
    return_address: usize,
    func_name: String,
}

impl CallFrame {
    fn new(stack_index: usize, return_address: usize, func_name: String) -> CallFrame {
        CallFrame {
            stack_index,
            return_address,
            func_name,
        }
    }
}

pub struct VM {
    stack: Vec<SquatValue>,
    call_stack: Vec<CallFrame>,
    globals: Vec<Option<SquatValue>>,
    natives: Vec<CompilerNative>,
    constants: ValueArray,
    current_chunk: usize,
    chunks: Vec<Chunk>,
    had_error: bool,
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
            had_error: false,
        }
    }

    pub fn interpret_source(&mut self, source: String, opts: &Options) -> InterpretResult {
        self.define_native_functions();
        let mut compiler = Compiler::new(
            &source,
            &mut self.chunks[0],
            &mut self.constants,
            &self.natives,
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
                self.call_stack.push(CallFrame::new(
                    0,
                    self.chunks[0].get_main_start(),
                    "main".to_owned(),
                ));

                self.interpret_chunk(0, opts)
            }
            CompileStatus::Fail => InterpretResult::InterpretCompileError,
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
                    }

                    OpCode::False => self.stack.push(SquatValue::Bool(false)),
                    OpCode::Nil => self.stack.push(SquatValue::Nil),
                    OpCode::True => self.stack.push(SquatValue::Bool(true)),

                    OpCode::Add => self.binary_op(|left, right| left + right),
                    OpCode::Subtract => self.binary_op(|left, right| left - right),
                    OpCode::Multiply => self.binary_op(|left, right| left * right),
                    OpCode::Divide => self.binary_op(|left, right| left / right),
                    OpCode::Mod => self.binary_op(|left, right| left % right),

                    OpCode::Equal => self.binary_cmp(|left, right| left == right),
                    OpCode::NotEqual => self.binary_cmp(|left, right| left != right),
                    OpCode::Greater => self.binary_cmp(|left, right| left > right),
                    OpCode::GreaterEqual => self.binary_cmp(|left, right| left >= right),
                    OpCode::Less => self.binary_cmp(|left, right| left < right),
                    OpCode::LessEqual => self.binary_cmp(|left, right| left <= right),

                    OpCode::Not => {
                        if let Some(value) = self.stack.pop() {
                            self.stack.push(SquatValue::Bool(!value.is_truthy()));
                        } else {
                            unreachable!("'!' cannot be used alone");
                        }
                    }
                    OpCode::Negate => match self.stack.pop() {
                        Some(SquatValue::Float(value)) => {
                            self.stack.push(SquatValue::Float(-value));
                        }
                        Some(SquatValue::Int(value)) => {
                            self.stack.push(SquatValue::Int(-value));
                        }
                        _ => unreachable!("Negate requires a number value"),
                    },

                    OpCode::Pop => {
                        self.stack.pop();
                    }

                    OpCode::DefineGlobal(index) => {
                        let index = *index;
                        if let Some(value) = self.stack.pop() {
                            self.globals[index] = Some(value);
                        } else {
                            unreachable!("DefineGlobal OpCode expects a value to be on the stack");
                        }
                    }
                    OpCode::GetGlobal(index) => {
                        let index = *index;
                        if let Some(Some(value)) = self.globals.get(index) {
                            self.stack.push(value.clone());
                        } else {
                            self.runtime_error(&format!(
                                "Variable with index {} is not defined",
                                index
                            ));
                        }
                    }
                    OpCode::SetGlobal(index) => {
                        let index = *index;
                        if let Some(value) = self.stack.last() {
                            if let Some(Some(_value)) = self.globals.get(index) {
                                self.globals[index] = Some(value.clone());
                            } else {
                                self.runtime_error(&format!(
                                    "You cannot set a global variable before defining it"
                                ));
                            }
                        } else {
                            unreachable!("SetGlobal OpCode expects a value to be on the stack");
                        }
                    }

                    OpCode::GetLocal(index) => {
                        let index = index + self.call_stack.last().unwrap().stack_index;
                        self.stack.push(self.stack[index].clone());
                    }
                    OpCode::SetLocal(index) => {
                        if let Some(value) = self.stack.last() {
                            let index = index + self.call_stack.last().unwrap().stack_index;
                            self.stack[index] = value.clone();
                        } else {
                            unreachable!("SetLocal OpCode expects a value to be on the stack");
                        }
                    }

                    OpCode::GetNative(index) => {
                        self.stack.push(self.natives[*index].get_value().clone());
                    }

                    OpCode::GetGlobalProperty(object_index, property_index) => {
                        if let Some(Some(SquatValue::Object(SquatObject::Instance(
                            instance_data,
                        )))) = self.globals.get(*object_index)
                        {
                            self.stack.push(instance_data.get_property(*property_index));
                        } else {
                            unreachable!(
                                "GetGlobalProperty expected a class instance at global position {}",
                                object_index
                            );
                        }
                    }
                    OpCode::GetLocalProperty(object_index, property_index) => {
                        let stack_index =
                            object_index + self.call_stack.last().unwrap().stack_index;
                        if let Some(SquatValue::Object(SquatObject::Instance(instance_data))) =
                            self.stack.get(stack_index)
                        {
                            self.stack.push(instance_data.get_property(*property_index));
                        } else {
                            unreachable!(
                                "GetLocalProperty expected a class instance at stack index {}",
                                stack_index
                            );
                        }
                    }
                    OpCode::GetProperty(property_index) => {
                        if let Some(SquatValue::Object(SquatObject::Instance(instance))) =
                            self.stack.pop()
                        {
                            self.stack.push(instance.get_property(*property_index));
                        } else {
                            unreachable!(
                                "GetProperty OpCode expects a class instance on top of the stack"
                            );
                        }
                    }
                    OpCode::SetGlobalProperty(object_index, property_index) => {
                        let object = &mut self.globals[*object_index];
                        if let Some(SquatValue::Object(SquatObject::Instance(instance_data))) =
                            object
                        {
                            instance_data
                                .set_property(*property_index, self.stack.last().unwrap().clone());
                        } else {
                            unreachable!(
                                "SetGlobalProperty expected a class instance at global position {}",
                                object_index
                            );
                        }
                    }
                    OpCode::SetLocalProperty(object_index, property_index) => {
                        let stack_index =
                            object_index + self.call_stack.last().unwrap().stack_index;
                        let value = self.stack.last().unwrap().clone();
                        let object = &mut self.stack[stack_index];
                        if let SquatValue::Object(SquatObject::Instance(instance_data)) = object {
                            instance_data.set_property(*property_index, value);
                        } else {
                            unreachable!(
                                "SetLocalProperty expected a class instance at global position {}",
                                object_index
                            );
                        }
                    }

                    OpCode::Index => {
                        if let Some(SquatValue::Int(index)) = self.stack.pop() {
                            if let Some(indexed_value) = self.stack.pop() {
                                match indexed_value {
                                    SquatValue::String(value) => {
                                        if value.len() as i64 <= index {
                                            self.runtime_error(&format!("Index out of range, max possible index is {} but {} was given", value.len() - 1, index));
                                        } else if index < 0 {
                                            self.runtime_error(&format!(
                                                "Given index {} is a negative number",
                                                index
                                            ));
                                        } else {
                                            self.stack.push(SquatValue::String(String::from(
                                                value.as_bytes()[index as usize] as char,
                                            )));
                                        }
                                    }
                                    _ => unreachable!(
                                        "Unexpected type on the stack for OpCode Index {:?}",
                                        indexed_value
                                    ),
                                }
                            } else {
                                unreachable!("Index OpCode expects a value on the stack after the index integer is poped")
                            }
                        } else {
                            unreachable!("Index OpCode expects an Int on top of the stack")
                        }
                    }

                    OpCode::JumpTo(instruction_number) => {
                        self.chunks[self.current_chunk].current_instruction = *instruction_number;
                    }
                    OpCode::JumpIfFalse(offset) => {
                        if let Some(value) = self.stack.last() {
                            if !value.is_truthy() {
                                self.chunks[self.current_chunk].current_instruction += *offset;
                            }
                        } else {
                            unreachable!("JumpIfFalse OpCode expect a value to be on the stack");
                        }
                    }
                    OpCode::Jump(offset) => {
                        self.chunks[self.current_chunk].current_instruction += *offset;
                    }
                    OpCode::JumpIfTrue(offset) => {
                        if let Some(value) = self.stack.last() {
                            if value.is_truthy() {
                                self.chunks[self.current_chunk].current_instruction += *offset;
                            }
                        } else {
                            unreachable!("JumpIfTrue OpCode expect a value to be on the stack");
                        }
                    }
                    OpCode::Loop(loop_start) => {
                        self.chunks[self.current_chunk].current_instruction = *loop_start;
                    }

                    OpCode::Call(arg_count) => {
                        let arg_count = *arg_count;
                        let func_data_location = self.stack.len() - 1 - arg_count;
                        // All this ugly code for the native stuff exists because of the
                        // borrow checker.
                        let native = match self.stack.get(func_data_location).unwrap() {
                            SquatValue::Object(SquatObject::Function(func_data)) => {
                                let return_address =
                                    self.chunks[self.current_chunk].current_instruction;
                                self.call_stack.push(CallFrame::new(
                                    self.stack.len() - arg_count,
                                    return_address,
                                    func_data.name.clone(),
                                ));
                                self.chunks[self.current_chunk].current_instruction =
                                    func_data.start_instruction_index;
                                continue;
                            }
                            SquatValue::Object(SquatObject::NativeFunction(func)) => func.clone(),
                            _ => unreachable!("Call OpCode expects a FunctionObject on the stack"),
                        };

                        let mut args = Vec::new();
                        for _i in 0..arg_count {
                            args.push(self.stack.pop().unwrap())
                        }
                        self.stack.pop().unwrap();
                        args.reverse();
                        match native.call(args) {
                            Ok(value) => self.stack.push(value),
                            Err(msg) => self.runtime_error(&msg),
                        };
                    }
                    OpCode::CreateInstance(arg_count) => {
                        let arg_count = *arg_count;
                        let class_data_location = self.stack.len() - 1 - arg_count;
                        match self.stack.get(class_data_location).unwrap() {
                            SquatValue::Object(SquatObject::Struct(_)) => {
                                let mut args = Vec::new();
                                for _i in 0..arg_count {
                                    args.push(self.stack.pop().unwrap())
                                }
                                match self.stack.pop() {
                                    Some(SquatValue::Object(SquatObject::Struct(class_data))) => {
                                        args.reverse();
                                        self.stack.push(SquatValue::Object(SquatObject::Instance(
                                            SquatInstance::new(&class_data.name, args),
                                        )));
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            _ => unreachable!("CreateInstace OpCode expects a Class on the stack"),
                        };
                    }
                    OpCode::Return => {
                        let return_val = self.stack.pop().unwrap();
                        if let Some(call_frame) = self.call_stack.pop() {
                            while call_frame.stack_index < self.stack.len() {
                                self.stack.pop(); // Pop local variables
                            }
                            self.stack.pop(); // Pop SquatFunc
                            self.chunks[self.current_chunk].current_instruction =
                                call_frame.return_address;
                            self.stack.push(return_val);
                        } else {
                            if let SquatValue::Int(i) = return_val {
                                return InterpretResult::InterpretOk(i);
                            }
                            return InterpretResult::InterpretOk(0);
                        }
                    }

                    OpCode::Start => {}
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
    where
        F: FnOnce(SquatValue, SquatValue) -> SquatValue,
    {
        let right = self.stack.pop();
        let left = self.stack.pop();

        if left.is_some() && right.is_some() {
            self.stack.push(op(left.unwrap(), right.unwrap()));
        } else {
            unreachable!("Binary operations require 2 values in the stack");
        }
    }

    fn binary_cmp<F>(&mut self, op: F)
    where
        F: FnOnce(SquatValue, SquatValue) -> bool,
    {
        let right = self.stack.pop();
        let left = self.stack.pop();

        if left.is_some() && right.is_some() {
            self.stack
                .push(SquatValue::Bool(op(left.unwrap(), right.unwrap())));
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
        self.define_native_func(
            "input",
            native::io::input,
            SquatFunctionTypeData::new(vec![], SquatType::String),
        );
        self.define_native_func(
            "print",
            native::io::print,
            SquatFunctionTypeData::new(vec![SquatType::Any], SquatType::Nil),
        );
        self.define_native_func(
            "println",
            native::io::println,
            SquatFunctionTypeData::new(vec![SquatType::Any], SquatType::Nil),
        );

        self.define_native_func(
            "cbrt",
            native::number::cbrt,
            SquatFunctionTypeData::new(vec![SquatType::Number], SquatType::Float),
        );
        self.define_native_func(
            "sqrt",
            native::number::sqrt,
            SquatFunctionTypeData::new(vec![SquatType::Number], SquatType::Float),
        );
        self.define_native_func(
            "pow",
            native::number::pow,
            SquatFunctionTypeData::new(
                vec![SquatType::Number, SquatType::Number],
                SquatType::Float,
            ),
        );
        self.define_native_func(
            "to_int",
            native::number::to_int,
            SquatFunctionTypeData::new(vec![SquatType::Any], SquatType::Int),
        );
        self.define_native_func(
            "to_float",
            native::number::to_float,
            SquatFunctionTypeData::new(vec![SquatType::Any], SquatType::Float),
        );

        self.define_native_func(
            "exit",
            native::misc::exit,
            SquatFunctionTypeData::new(vec![SquatType::Int], SquatType::Nil),
        );
        self.define_native_func(
            "time",
            native::misc::time,
            SquatFunctionTypeData::new(vec![], SquatType::Float),
        );
        self.define_native_func(
            "type",
            native::misc::get_type,
            SquatFunctionTypeData::new(vec![SquatType::Any], SquatType::Type),
        );

        self.define_native_func(
            "to_str",
            native::string::to_str,
            SquatFunctionTypeData::new(vec![SquatType::Any], SquatType::String),
        );
    }

    fn define_native_func(
        &mut self,
        name: &str,
        func: native::NativeFunc,
        func_data: SquatFunctionTypeData,
    ) {
        let native_func = SquatNativeFunction::new(name, func);
        let native_object = SquatObject::NativeFunction(native_func);
        let native_value = SquatValue::Object(native_object);

        let native_compiler: CompilerNative =
            CompilerNative::new(native_value, SquatType::NativeFunction(func_data));
        self.natives.push(native_compiler);
    }
}
