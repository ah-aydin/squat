#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Constant(usize),
    
    False, Nil, True,

    Add, Subtract, Multiply, Divide, Mod,

    Concat, 

    Equal, NotEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Not, Negate,

    Print, Pop,

    DefineGlobal(usize), GetGlobal(usize), SetGlobal(usize),
    GetLocal(usize), SetLocal(usize),

    JumpTo(usize), JumpIfFalse(usize), Jump(usize), JumpIfTrue(usize),
    Loop(usize),

    Call(usize, usize), // (Instruction, ArgCount)
    Return,

    Start, Stop
}
