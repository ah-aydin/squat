use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SquatInstanceTypeData {
    pub struct_name: String,
}

impl SquatInstanceTypeData {
    pub fn new(struct_name: &str) -> SquatInstanceTypeData {
        SquatInstanceTypeData {
            struct_name: struct_name.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SquatStructTypeData {
    pub name: String,
    field_types: Vec<SquatType>,
    fields: HashMap<String, (SquatType, usize)>,
}

impl SquatStructTypeData {
    pub fn new(name: &str) -> SquatStructTypeData {
        SquatStructTypeData {
            name: name.to_string(),
            field_types: vec![],
            fields: HashMap::new(),
        }
    }

    pub fn get_instance_type(&self) -> SquatType {
        SquatType::Instance(SquatInstanceTypeData::new(&self.name))
    }

    pub fn get_field_type_by_index(&self, field_index: usize) -> SquatType {
        match self.field_types.get(field_index) {
            Some(field_type) => field_type.clone(),
            None => {
                unreachable!("{} {:?}", field_index, self.field_types)
            }
        }
    }

    pub fn get_field_type_and_index_by_name(
        &self,
        field_name: &str,
    ) -> Result<(SquatType, usize), ()> {
        match self.fields.get(field_name) {
            Some((field_type, index)) => Ok((field_type.clone(), *index)),
            None => Err(()),
        }
    }

    pub fn add_field(&mut self, field_name: &str, field_type: SquatType) {
        self.field_types.push(field_type.clone());
        self.fields.insert(
            field_name.to_owned(),
            (field_type, self.field_types.len() - 1),
        );
    }

    pub fn get_field_count(&self) -> usize {
        self.field_types.len()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SquatFunctionTypeData {
    pub param_types: Vec<SquatType>,
    return_type: Box<SquatType>,
}

impl SquatFunctionTypeData {
    pub fn new(param_types: Vec<SquatType>, return_type: SquatType) -> SquatFunctionTypeData {
        SquatFunctionTypeData {
            param_types,
            return_type: Box::new(return_type),
        }
    }

    pub fn get_return_type(&self) -> SquatType {
        *self.return_type.clone()
    }

    pub fn set_return_type(&mut self, return_type: SquatType) {
        self.return_type = Box::new(return_type);
    }

    pub fn get_param_type(&self, arg_count: usize) -> SquatType {
        match self.param_types.get(arg_count) {
            Some(param_type) => param_type.clone(),
            None => {
                unreachable!("{} {:?}", arg_count, self.param_types)
            }
        }
    }

    pub fn get_arity(&self) -> usize {
        self.param_types.len()
    }
}

impl PartialEq for SquatFunctionTypeData {
    fn eq(&self, other: &Self) -> bool {
        return self.param_types == other.param_types
            && self.get_return_type() == other.get_return_type();
    }
}

#[derive(Debug, Clone)]
pub enum SquatType {
    Nil,
    Int,
    Float,
    String,
    Bool,
    Function(SquatFunctionTypeData),
    NativeFunction(SquatFunctionTypeData),
    Struct(SquatStructTypeData),
    Instance(SquatInstanceTypeData),
    Type,
    Number,
    Any,
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
                data.param_types
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                data.get_return_type()
            ),
            SquatType::NativeFunction(data) => write!(
                f,
                "<type NativeFunction ({}) {}>",
                data.param_types
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                data.get_return_type()
            ),
            SquatType::Struct(data) => write!(f, "<type Struct {}>", data.name),
            SquatType::Instance(data) => write!(f, "<type Instance of {}>", data.struct_name),
            SquatType::Type => write!(f, "<type Type>"),
            SquatType::Any => write!(f, "<type Any>"),
            SquatType::Number => write!(f, "<type Number>"),
        }
    }
}

impl PartialEq for SquatType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SquatType::Nil, SquatType::Nil)
            | (SquatType::Int, SquatType::Int)
            | (SquatType::Float, SquatType::Float)
            | (SquatType::Bool, SquatType::Bool)
            | (SquatType::Type, SquatType::Type)
            | (SquatType::String, SquatType::String)
            | (SquatType::Any, _)
            | (_, SquatType::Any)
            | (SquatType::Number, SquatType::Number)
            | (SquatType::Number, SquatType::Int)
            | (SquatType::Number, SquatType::Float)
            | (SquatType::Int, SquatType::Number)
            | (SquatType::Float, SquatType::Number) => true,
            (SquatType::Function(data), SquatType::Function(data2))
            | (SquatType::NativeFunction(data), SquatType::NativeFunction(data2)) => data == data2,
            (SquatType::Struct(data), SquatType::Struct(data2)) => data == data2,
            (SquatType::Instance(data), SquatType::Instance(data2)) => data == data2,
            (_, _) => false,
        }
    }
}
