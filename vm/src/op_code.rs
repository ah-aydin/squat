#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Constant(usize),
    
    False, Nil, True,

    Add, Subtract, Multiply, Divide, Mod,

    Equal, NotEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Not, Negate,

    Pop,

    DefineGlobal(usize), GetGlobal(usize), SetGlobal(usize),
    GetLocal(usize), SetLocal(usize),
    GetNative(usize),

    JumpTo(usize), JumpIfFalse(usize), Jump(usize), JumpIfTrue(usize),
    Loop(usize),

    Call(usize),
    Return,

    Start, Stop,
}
