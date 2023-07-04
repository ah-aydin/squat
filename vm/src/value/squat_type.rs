use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SquatType {
    Nil,
    Int,
    Float,
    String,
    Bool,
    Function,
    NativeFunction,
    Type
}

impl fmt::Display for SquatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquatType::Nil => write!(f, "Nil"),
            SquatType::Int => write!(f, "Int"),
            SquatType::Float => write!(f, "Float"),
            SquatType::String => write!(f, "String"),
            SquatType::Bool => write!(f, "Bool"),
            SquatType::Function => write!(f, "Function"),
            SquatType::NativeFunction=> write!(f, "NativeFunction"),
            SquatType::Type => write!(f, "Type"),
        }
    }
}

