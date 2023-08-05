use crate::value::{squat_type::SquatType, squat_value::SquatValue};

#[derive(Debug)]
pub struct CompilerLocal {
    pub name: String,
    // If this value is missing, the variable is not initialized yet.
    pub depth: Option<u32>,
    squat_type: Option<SquatType>,
}

impl CompilerLocal {
    pub fn new(name: &str, depth: Option<u32>, squat_type: Option<SquatType>) -> CompilerLocal {
        CompilerLocal {
            name: name.to_string(),
            depth,
            squat_type,
        }
    }

    pub fn get_type(&self) -> SquatType {
        self.squat_type.as_ref().unwrap_or(&SquatType::Nil).clone()
    }

    pub fn set_type(&mut self, squat_type: SquatType) {
        self.squat_type = Some(squat_type);
    }
}

#[derive(Debug)]
pub struct CompilerNative {
    squat_value: SquatValue,
    squat_type: SquatType,
}

impl CompilerNative {
    pub fn new(squat_value: SquatValue, squat_type: SquatType) -> CompilerNative {
        CompilerNative {
            squat_value,
            squat_type,
        }
    }

    pub fn get_value(&self) -> SquatValue {
        self.squat_value.clone()
    }

    pub fn get_type(&self) -> SquatType {
        self.squat_type.clone()
    }
}

#[derive(Debug)]
pub struct CompilerGlobal {
    pub index: usize,
    pub initialized: bool,
    squat_type: Option<SquatType>,
}

impl CompilerGlobal {
    pub fn new(index: usize, initialized: bool, squat_type: Option<SquatType>) -> CompilerGlobal {
        CompilerGlobal {
            index,
            initialized,
            squat_type,
        }
    }

    pub fn get_type(&self) -> SquatType {
        self.squat_type.as_ref().unwrap_or(&SquatType::Nil).clone()
    }

    pub fn set_type(&mut self, squat_type: SquatType) {
        self.squat_type = Some(squat_type);
    }
}
