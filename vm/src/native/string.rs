use super::*;
use crate::value::squat_value::SquatValue;

pub fn to_str(args: NativeFuncArgs) -> NativeFuncReturnType {
    Ok(match &args[0] {
        SquatValue::Nil => SquatValue::String("Nil".to_owned()),
        SquatValue::Int(value) => SquatValue::String(value.to_string()),
        SquatValue::Float(value) => SquatValue::String(value.to_string()),
        SquatValue::String(value) => SquatValue::String(value.to_string()),
        SquatValue::Bool(value) => SquatValue::String(value.to_string()),
        SquatValue::Object(value) => SquatValue::String(value.to_string()),
        SquatValue::Type(value) => SquatValue::String(value.to_string()),
    })
}
