use std::fmt;

use super::squat_value::SquatValue;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SquatNativeFunctionTypeData {
    pub arity: usize,
    pub param_types: Vec<Vec<SquatType>>,
    return_type: Box<SquatType>
}

impl SquatNativeFunctionTypeData {
    pub fn new(
        arity: usize,
        param_types: Vec<Vec<SquatType>>,
        return_type: SquatType
    ) -> SquatNativeFunctionTypeData {
        SquatNativeFunctionTypeData {
            arity,
            param_types,
            return_type: Box::new(return_type)
        }
    }

    pub fn get_return_type(&self) -> SquatType {
        *self.return_type.clone()
    }

    pub fn get_param_type(&self, arg_count: usize) -> Vec<SquatType> {
        self.param_types.get(arg_count).unwrap().clone()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SquatFunctionTypeData {
    pub arity: usize,
    pub param_types: Vec<SquatType>,
    return_type: Box<SquatType>
}

impl SquatFunctionTypeData {
    pub fn new(
        arity: usize,
        param_types: Vec<SquatType>,
        return_type: SquatType
    ) -> SquatFunctionTypeData {
        SquatFunctionTypeData {
            arity,
            param_types,
            return_type: Box::new(return_type)
        }
    }

    pub fn get_return_type(&self) -> SquatType {
        *self.return_type.clone()
    }

    pub fn get_param_type(&self, arg_count: usize) -> SquatType {
        self.param_types.get(arg_count).unwrap().clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SquatType {
    Nil,
    Int,
    Float,
    String,
    Bool,
    Function(SquatFunctionTypeData),
    NativeFunction,
    Type,
    Any // Used only in native function calls
}

impl Default for SquatType {
    fn default() -> Self {
        SquatType::Nil
    }
}

impl fmt::Display for SquatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquatType::Nil => write!(f, "<type Nil>"),
            SquatType::Int => write!(f, "<type Int>"),
            SquatType::Float => write!(f, "<type Float>"),
            SquatType::String => write!(f, "<type String>"),
            SquatType::Bool => write!(f, "<type Bool>"),
            SquatType::Function(_) => write!(f, "<type Function>"),
            SquatType::NativeFunction=> write!(f, "<type NativeFunction>"),
            SquatType::Type => write!(f, "<type Type>"),
            SquatType::Any => write!(f, "<type Any>"),
        }
    }
}

