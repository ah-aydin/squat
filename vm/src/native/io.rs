use super::*;
use crate::value::squat_value::SquatValue;

pub fn print(args: NativeFuncArgs) -> NativeFuncReturnType {
    let output = args
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    print!("{}", output);
    Ok(SquatValue::Nil)
}

pub fn println(args: NativeFuncArgs) -> NativeFuncReturnType {
    let output = args
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(" ");
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
        }
        Err(msg) => Err(msg.to_string()),
    }
}
