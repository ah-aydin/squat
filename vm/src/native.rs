use crate::value::squat_value::SquatValue;

pub mod io;
pub mod misc;
pub mod number;
pub mod string;

pub type NativeFuncArgs = Vec<SquatValue>;
pub type NativeFuncReturnType = Result<SquatValue, String>;
pub type NativeFunc = fn(NativeFuncArgs) -> NativeFuncReturnType;
