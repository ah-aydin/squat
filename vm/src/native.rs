use crate::value::SquatValue;

pub mod io;
pub mod number;
pub mod misc;

pub type NativeFuncArgs = Vec<SquatValue>;
pub type NativeFuncReturnType = Result<SquatValue, String>;
pub type NativeFunc = fn(NativeFuncArgs) -> NativeFuncReturnType;
