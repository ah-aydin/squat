use std::time::{SystemTime, UNIX_EPOCH};

use crate::{value::SquatValue, object::NativeFuncArgs};

pub fn time(_args: NativeFuncArgs) -> SquatValue {
    let now = SystemTime::now();
    let value = now.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs_f64();
    SquatValue::Number(value)
}
