use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SquatType {
    Nil,
    Int,
    Float,
    String,
    Bool,
    Function,
}

impl fmt::Display for SquatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquatType::Nil => write!(f, "<type Nil>"),
            SquatType::Int => write!(f, "<type Int>"),
            SquatType::Float => write!(f, "<type Float>"),
            SquatType::String => write!(f, "<type String>"),
            SquatType::Bool => write!(f, "<type Bool>"),
            SquatType::Function => write!(f, "<type Function>"),
        }
    }
}

