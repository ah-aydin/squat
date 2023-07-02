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
    let output = args.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ");
    print!("{}", output);
    Ok(SquatValue::Nil)
}

pub fn println(args: NativeFuncArgs) -> NativeFuncReturnType {
    let output = args.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ");
    println!("{}", output);
    Ok(SquatValue::Nil)
}
