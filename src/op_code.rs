#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
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

    JumpTo(usize), JumpOffset(usize), JumpIfFalse, Jump, JumpIfTrue, JumpBack, Loop,

    Return,

    Start, Stop
}
