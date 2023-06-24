use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SquatValue {
    Nil,
    Number(f64),
    String(String),
    Bool(bool),
}

impl SquatValue {
    pub fn to_string(&self) -> String {
        match self {
            SquatValue::Nil => String::from("nil"),
            SquatValue::Number(value) => value.to_string(),
            SquatValue::String(value) => value.clone(),
            SquatValue::Bool(true) => "true".to_owned(),
            SquatValue::Bool(false) => "false".to_owned(),
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
            SquatValue::String(value)   => write!(f, "{}", value)
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
        // TODO consider storing the indicies in a hash map in the future if things get too slow
        // It might be faster, for now with small programs, linear search should be faster then
        // computing the hash
        if let Some(index) = self.values.iter().position(|v| *v == value) {
            return index;
        }
        self.values.push(value);
        self.values.len() - 1
    }
}
