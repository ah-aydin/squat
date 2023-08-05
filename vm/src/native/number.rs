use super::*;
use crate::value::squat_value::SquatValue;

pub fn cbrt(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::Float(value) => Ok(SquatValue::Float(value.cbrt())),
        SquatValue::Int(value) => Ok(SquatValue::Float((*value as f64).cbrt())),
        _ => Err(format!("'{}' is not of type number", args[0])),
    }
}

pub fn sqrt(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::Float(value) => Ok(SquatValue::Float(value.sqrt())),
        SquatValue::Int(value) => Ok(SquatValue::Float((*value as f64).sqrt())),
        _ => Err(format!("'{}' is not of type number", args[0])),
    }
}

pub fn pow(args: NativeFuncArgs) -> NativeFuncReturnType {
    match (&args[0], &args[1]) {
        (SquatValue::Float(value), SquatValue::Float(power)) => {
            Ok(SquatValue::Float(value.powf(*power)))
        }
        (SquatValue::Int(value), SquatValue::Int(power)) => {
            Ok(SquatValue::Float(value.pow(*power as u32) as f64))
        }
        (SquatValue::Int(value), SquatValue::Float(power)) => {
            Ok(SquatValue::Float((*value as f64).powf(*power)))
        }
        (SquatValue::Float(value), SquatValue::Int(power)) => {
            Ok(SquatValue::Float(value.powf(*power as f64)))
        }
        _ => Err(format!("'{}' is not of type number", args[0])),
    }
}

pub fn to_int(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::String(value) => match value.parse::<i64>() {
            Ok(value) => Ok(SquatValue::Int(value)),
            Err(_) => Err(format!("Can't cast '{}' to a number", args[0])),
        },
        SquatValue::Float(value) => Ok(SquatValue::Int(*value as i64)),
        SquatValue::Bool(true) => Ok(SquatValue::Int(1)),
        SquatValue::Bool(false) => Ok(SquatValue::Int(0)),
        SquatValue::Int(value) => Ok(SquatValue::Int(*value)),
        _ => Err(format!("Can't cast '{}' to an int", args[0])),
    }
}

pub fn to_float(args: NativeFuncArgs) -> NativeFuncReturnType {
    match &args[0] {
        SquatValue::String(value) => match value.parse::<f64>() {
            Ok(value) => Ok(SquatValue::Float(value)),
            Err(_) => Err(format!("Can't cast '{}' to a float", args[0])),
        },
        SquatValue::Int(value) => Ok(SquatValue::Float(*value as f64)),
        SquatValue::Bool(true) => Ok(SquatValue::Float(1.)),
        SquatValue::Bool(false) => Ok(SquatValue::Float(0.)),
        SquatValue::Float(value) => Ok(SquatValue::Float(*value)),
        _ => Err(format!("Can't cast '{}' to a number", args[0])),
    }
}
