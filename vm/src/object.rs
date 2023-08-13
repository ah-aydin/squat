use crate::{
    native::{NativeFunc, NativeFuncArgs, NativeFuncReturnType},
    value::{squat_type::SquatType, squat_value::SquatValue},
};

#[derive(Debug, Clone, Default)]
pub struct SquatClass {
    pub name: String,
}
impl SquatClass {
    pub fn new(name: &str) -> SquatClass {
        SquatClass {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SquatInstance {
    pub instance_of: String,
    properties: Vec<SquatValue>,
}
impl SquatInstance {
    pub fn new(instance_of: &str, properties: Vec<SquatValue>) -> SquatInstance {
        SquatInstance {
            instance_of: instance_of.to_string(),
            properties,
        }
    }

    pub fn get_property(&self, index: usize) -> SquatValue {
        self.properties.get(index).unwrap().clone()
    }

    pub fn set_property(&mut self, index: usize, value: SquatValue) {
        self.properties[index] = value;
    }
}

impl PartialEq for SquatInstance {
    fn eq(&self, other: &Self) -> bool {
        if self.instance_of == other.instance_of {
            return self.properties == other.properties;
        }
        false
    }
}

#[derive(Debug, Clone, Default)]
pub struct SquatFunction {
    pub name: String,
    pub start_instruction_index: usize,
}

impl SquatFunction {
    pub fn new(name: &str, start_instruction_index: usize) -> SquatFunction {
        SquatFunction {
            name: name.to_owned(),
            start_instruction_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SquatNativeFunction {
    pub name: String,
    function: NativeFunc,
}

impl SquatNativeFunction {
    pub fn new(name: &str, function: NativeFunc) -> SquatNativeFunction {
        SquatNativeFunction {
            name: name.to_string(),
            function,
        }
    }

    pub fn call(&self, args: NativeFuncArgs) -> NativeFuncReturnType {
        (self.function)(args)
    }
}

#[derive(Debug, Clone)]
pub enum SquatObject {
    Function(SquatFunction),
    NativeFunction(SquatNativeFunction),
    Class(SquatClass),
    Instance(SquatInstance),
}

impl SquatObject {
    pub fn get_type(&self) -> SquatType {
        match self {
            SquatObject::Function(_) => SquatType::Function(Default::default()),
            SquatObject::NativeFunction(_) => SquatType::NativeFunction(Default::default()),
            SquatObject::Class(_) => SquatType::Class(Default::default()),
            SquatObject::Instance(_) => SquatType::Instance(Default::default()),
        }
    }
}

impl ToString for SquatObject {
    fn to_string(&self) -> String {
        match self {
            SquatObject::Function(func) => format!("<func {}>", func.name),
            SquatObject::NativeFunction(func) => format!("<native func {}>", func.name),
            SquatObject::Class(class) => format!("<class {}>", class.name),
            SquatObject::Instance(instance) => format!(
                "<instance of {} {:?}>",
                instance.instance_of, instance.properties
            ),
        }
    }
}

impl PartialEq for SquatObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SquatObject::Function(func1), SquatObject::Function(func2)) => {
                func1.start_instruction_index == func2.start_instruction_index
            }
            (SquatObject::NativeFunction(func1), SquatObject::NativeFunction(func2)) => {
                func1.name == func2.name
            }
            (SquatObject::Class(class1), SquatObject::Class(class2)) => class1.name == class2.name,
            (SquatObject::Instance(instance1), SquatObject::Instance(instance2)) => {
                instance1 == instance2
            }
            _ => false,
        }
    }
}
