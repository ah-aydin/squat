use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    object::{
        NativeFuncArgs,
        NativeFuncReturnType
    },
    value::SquatValue
};

pub fn time(_args: NativeFuncArgs) -> NativeFuncReturnType {
    let now = SystemTime::now();
    let value = now.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs_f64();
    Ok(SquatValue::Number(value))
}
