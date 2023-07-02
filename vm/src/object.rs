use crate::value::SquatValue;

pub type NativeFuncArgs = Vec<SquatValue>;
pub type NativeFuncReturnType = SquatValue;
pub type NativeFunc = fn(NativeFuncArgs) -> SquatValue;

#[derive(Debug, Clone)]
pub struct SquatFunction {
    pub name: String,
    pub start_instruction_index: usize,
    pub arity: usize
}

impl SquatFunction {
    pub fn new(name: &str, start_instruction_index: usize, arity: usize) -> SquatFunction {
        SquatFunction {
            name: name.to_owned(),
            start_instruction_index,
            arity
        }
    }
}

#[derive(Debug, Clone)]
pub struct SquatNativeFunction {
    pub name: String,
    pub arity: usize,
    function: NativeFunc
}

impl SquatNativeFunction {
    pub fn new(name: &str, arity: usize, function: NativeFunc) -> SquatNativeFunction {
        SquatNativeFunction { name: name.to_string(), arity , function }
    }

    pub fn call(&self, args: NativeFuncArgs) -> SquatValue {
        (self.function)(args)
    }
}

#[derive(Debug, Clone)]
pub enum SquatObject {
    Function(SquatFunction),
    NativeFunction(SquatNativeFunction)
}

impl ToString for SquatObject {
    fn to_string(&self) -> String {
        match self {
            SquatObject::Function(func) => format!("<func {}>", func.name),
            SquatObject::NativeFunction(func) => format!("<native func {}>", func.name)
        }
    }
}

impl PartialEq for SquatObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                SquatObject::Function(func1), 
                SquatObject::Function(func2)
            ) => func1.start_instruction_index == func2.start_instruction_index,
            (
                SquatObject::NativeFunction(func1),
                SquatObject::NativeFunction(func2)
            ) => func1.name == func2.name,
            _ => false
        }
    }
}
