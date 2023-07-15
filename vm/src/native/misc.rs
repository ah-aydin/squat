use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn exit(args: NativeFuncArgs) -> NativeFuncReturnType {
    let exit_code: SquatValue = args[0].clone();
    if let SquatValue::Int(exit_code) = exit_code {
        let exit_code: i32 = exit_code as i32;
        std::process::exit(exit_code);
    } else {
        std::process::exit(-1);
    }
}

pub fn time(_args: NativeFuncArgs) -> NativeFuncReturnType {
    let now = SystemTime::now();
    let value = now.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs_f64();
    Ok(SquatValue::Float(value))
}

pub fn get_type(args: NativeFuncArgs) -> NativeFuncReturnType {
    Ok(SquatValue::Type(args[0].get_type()))
}
