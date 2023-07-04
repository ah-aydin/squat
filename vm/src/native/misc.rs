use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn time(_args: NativeFuncArgs) -> NativeFuncReturnType {
    let now = SystemTime::now();
    let value = now.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs_f64();
    Ok(SquatValue::Float(value))
}

pub fn get_type(args: NativeFuncArgs) -> NativeFuncReturnType {
    Ok(SquatValue::Type(args[0].get_type()))
}
