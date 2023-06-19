#[derive(Debug, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum OpCode {
    Constant, Index(usize),
    
    False, Nil, True,

    Add, Subtract, Multiply, Divide,

    Concat, 

    Equal, NotEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Not, Negate,

    Print, Pop,

    DefineGlobal, GetGlobal, SetGlobal,
    GetLocal, SetLocal,

    Return
}
