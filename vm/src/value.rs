use std::fmt;

use crate::object::SquatObject;

#[derive(Debug, Clone, PartialEq)]
pub enum SquatValue {
    Nil,
    Number(f64),
    String(String),
    Bool(bool),
    Object(SquatObject)
}

impl SquatValue {
    pub fn to_string(&self) -> String {
        match self {
            SquatValue::Nil => String::from("nil"),
            SquatValue::Number(value) => value.to_string(),
            SquatValue::String(value) => value.clone(),
            SquatValue::Bool(true) => "true".to_owned(),
            SquatValue::Bool(false) => "false".to_owned(),
            SquatValue::Object(object) => object.to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            SquatValue::Bool(true) => true,
            SquatValue::Bool(false) | SquatValue::Nil => false,
            _ => true
        }
    }
}

impl PartialOrd for SquatValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SquatValue::Nil, SquatValue::Nil) => Some(std::cmp::Ordering::Equal),
            (SquatValue::Number(f1), SquatValue::Number(f2)) => f1.partial_cmp(f2),
            (SquatValue::String(s1), SquatValue::String(s2)) => s1.partial_cmp(s2),
            _ => None
        }
    }
}

impl fmt::Display for SquatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquatValue::Nil             => write!(f, "Nil"),
            SquatValue::Number(value)   => write!(f, "{}", value),
            SquatValue::Bool(value)     => write!(f, "{}", value),
            SquatValue::String(value)   => write!(f, "{}", value),
            SquatValue::Object(object)  => {
                match object {
                    SquatObject::Function(func) => {
                        write!(f, "<fn {}>", func.name)
                    }
                }
            }
        }
    }
}

impl Default for SquatValue {
    fn default() -> Self {
        SquatValue::Nil
    }
}

#[derive(Debug)]
pub struct ValueArray {
    name: String,
    values: Vec<SquatValue>
}

impl ValueArray {
    pub fn new(name: &str) -> ValueArray {
        ValueArray {
            name: String::from(name),
            values: Vec::new()
        }
    }

    pub fn get(&self, index: usize) -> &SquatValue {
        if index >= self.values.len() {
            panic!("{} is out of range on ValueArray {}", index, self.name);
        }
        &self.values[index]
    }

    pub fn write(&mut self, value: SquatValue) -> usize {
        if let Some(index) = self.values.iter().position(|v| *v == value) {
            if let Some(SquatValue::Object(SquatObject::Function(func))) = self.values.get(index) {
                println!("Found same squat function {}", func.name);
            }
            return index;
        }
        self.values.push(value);
        self.values.len() - 1
    }
}
