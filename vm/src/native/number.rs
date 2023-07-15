use super::*;
use crate::value::squat_value::SquatValue;

pub fn cbrt(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::Float(value) => Ok(SquatValue::Float(value.cbrt())),
        _ => Err(format!("'{}' is not of type number", args[0]))
    }
}

pub fn sqrt(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::Float(value) => Ok(SquatValue::Float(value.sqrt())),
        _ => Err(format!("'{}' is not of type number", args[0]))
    }
}

pub fn pow(args: NativeFuncArgs) -> NativeFuncReturnType {
    match (&args[0], &args[1]) {
        (SquatValue::Float(value), SquatValue::Float(power))
            => Ok(SquatValue::Float(value.powf(*power))),
        _ => Err(format!("'{}' is not of type number", args[0]))
    }
}

pub fn to_int(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::String(value) => {
            match value.parse::<i64>() {
                Ok(value) => Ok(SquatValue::Int(value)),
                Err(_) => Err(format!("Can't cast '{}' to a number", args[0])),
            }
        },
        SquatValue::Float(value) => Ok(SquatValue::Int(*value as i64)),
        SquatValue::Bool(true) => Ok(SquatValue::Int(1)),
        SquatValue::Bool(false) => Ok(SquatValue::Int(0)),
        _ => Err(format!("Can't cast '{}' to a number", args[0]))
    }
}
