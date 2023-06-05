#[derive(Debug, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Index(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return
}
