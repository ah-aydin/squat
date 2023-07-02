use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    value::SquatValue,
    object::{
        NativeFuncArgs,
        NativeFuncReturnType
    }
};

pub fn time(_args: NativeFuncArgs) -> NativeFuncReturnType {
    let now = SystemTime::now();
    let value = now.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs_f64();
    Ok(SquatValue::Number(value))
}

pub fn print(args: NativeFuncArgs) -> NativeFuncReturnType {
    print!("{}", args[0]);
    Ok(SquatValue::Nil)
}

pub fn println(args: NativeFuncArgs) -> NativeFuncReturnType {
    println!("{}", args[0]);
    Ok(SquatValue::Nil)
}
