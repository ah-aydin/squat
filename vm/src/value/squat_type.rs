use std::fmt;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SquatNativeFunctionTypeData {
    pub arity: usize,
    pub param_types: Vec<Vec<SquatType>>,
    return_type: Box<SquatType>
}

impl SquatNativeFunctionTypeData {
}

#[derive(Debug, Clone, Default)]
pub struct SquatFunctionTypeData {
    pub arity: usize,
    pub param_types: Vec<SquatType>,
    return_type: Box<SquatType>
}

impl SquatFunctionTypeData {
    pub fn new(
        param_types: Vec<SquatType>,
        return_type: SquatType
    ) -> SquatFunctionTypeData {
        SquatFunctionTypeData {
            arity: param_types.len(),
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

impl PartialEq for SquatFunctionTypeData {
    fn eq(&self, other: &Self) -> bool {
        return self.param_types == other.param_types
            && self.get_return_type() == other.get_return_type();
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
    Class,
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
                "<type Function ({}) {}>",
                data
                    .param_types
                    .iter()
                    .map(|x| x.to_string()).
                    collect::<Vec<String>>().
                    join(" "),
                data.get_return_type()
            ),
            SquatType::NativeFunction=> write!(f, "<type NativeFunction>"),
            SquatType::Class => write!(f, "<type Class>"),
            SquatType::Type => write!(f, "<type Type>"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_function_type_equality() {
        let func1_type = SquatType::Function(SquatFunctionTypeData::new(vec![], SquatType::Nil));
        let func2_type = SquatType::Function(SquatFunctionTypeData::new(vec![], SquatType::Nil));
        assert_eq!(func1_type, func2_type);

        let func1_type = SquatType::Function(SquatFunctionTypeData::new(vec![], SquatType::Int));
        let func2_type = SquatType::Function(SquatFunctionTypeData::new(vec![], SquatType::Int));
        assert_eq!(func1_type, func2_type);

        let func1_type = SquatType::Function(SquatFunctionTypeData::new(vec![SquatType::Int], SquatType::Int));
        let func2_type = SquatType::Function(SquatFunctionTypeData::new(vec![SquatType::Int], SquatType::Int));
        assert_eq!(func1_type, func2_type);

        let func1_type = SquatType::Function(SquatFunctionTypeData::new(vec![SquatType::Int, SquatType::String], SquatType::Int));
        let func2_type = SquatType::Function(SquatFunctionTypeData::new(vec![SquatType::Int, SquatType::String], SquatType::Int));
        assert_eq!(func1_type, func2_type);
    }
}
