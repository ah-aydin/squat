#[derive(Debug, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum OpCode {
    Constant, Index(usize),
    
    False, Nil, True,

    Equal, NotEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Add, Subtract, Multiply, Divide,

    Not, Negate,

    Return
}
