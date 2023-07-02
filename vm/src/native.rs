use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    value::SquatValue,
    object::{
        NativeFuncArgs,
        NativeFuncReturnType
    }
};

/////////////////////////////////////////////////////////////
/// I/O
/////////////////////////////////////////////////////////////

pub fn number(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::String(value) => {
            match value.parse::<f64>() {
                Ok(value) => Ok(SquatValue::Number(value)),
                Err(_) => Err(format!("Can't cast '{}' to a number", args[0])),
            }
        },
        SquatValue::Number(value) => Ok(SquatValue::Number(*value)),
        SquatValue::Bool(true) => Ok(SquatValue::Number(1.)),
        SquatValue::Bool(false) => Ok(SquatValue::Number(0.)),
        _ => Err(format!("Can't cast '{}' to a number", args[0]))
    }
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

pub fn input(_args: NativeFuncArgs) -> NativeFuncReturnType {
    let mut value = String::new();
    match std::io::stdin().read_line(&mut value) {
        Ok(_) => {
            if value.ends_with('\n') {
                value.pop();
                if value.ends_with('\r') {
                    value.pop();
                }
            }
            Ok(SquatValue::String(value))
        },
        Err(msg) => Err(msg.to_string()),
    }
}

pub fn time(_args: NativeFuncArgs) -> NativeFuncReturnType {
    let now = SystemTime::now();
    let value = now.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs_f64();
    Ok(SquatValue::Number(value))
}
