use std::fmt;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SquatNativeFunctionTypeData {
    pub arity: usize,
    pub param_types: Vec<Vec<SquatType>>,
    return_type: Box<SquatType>
}

impl SquatNativeFunctionTypeData {
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

    pub fn set_return_type(&mut self, return_type: SquatType) {
        self.return_type = Box::new(return_type);
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
            SquatType::Function(data) => write!(
                f,
                "<type Function ({})>",
                data
                    .param_types
                    .iter()
                    .map(|x| x.to_string()).
                    collect::<Vec<String>>().
                    join(" ")
            ),
            SquatType::NativeFunction=> write!(f, "<type NativeFunction>"),
            SquatType::Type => write!(f, "<type Type>"),
        }
    }
}

