use super::*;
use crate::value::SquatValue;

pub fn cbrt(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::Number(value) => Ok(SquatValue::Number(value.cbrt())),
        _ => Err(format!("'{}' is not of type number", args[0]))
    }
}

pub fn sqrt(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::Number(value) => Ok(SquatValue::Number(value.sqrt())),
        _ => Err(format!("'{}' is not of type number", args[0]))
    }
}

pub fn pow(args: NativeFuncArgs) -> NativeFuncReturnType {
    match (&args[0], &args[1]) {
        (SquatValue::Number(value), SquatValue::Number(power))
            => Ok(SquatValue::Number(value.powf(*power))),
        _ => Err(format!("'{}' is not of type number", args[0]))
    }
}

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
