#[derive(Debug, Clone, PartialEq)]
pub enum SquatValue {
    Nil,
    Number(f64),
    String(String),
    Bool(bool),
}

impl PartialOrd for SquatValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SquatValue::Number(f1), SquatValue::Number(f2)) => f1.partial_cmp(f2),
            (SquatValue::String(s1), SquatValue::String(s2)) => s1.partial_cmp(s2),
            (SquatValue::Nil, SquatValue::Nil) => Some(std::cmp::Ordering::Equal),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct ValueArray {
    name: String,
    values: Vec<SquatValue>,
}

impl ValueArray {
    pub fn new(name: String) -> ValueArray {
        ValueArray {
            name: String::from(name),
            values: Vec::new()
        }
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> &SquatValue {
        if index >= self.values.len() {
            panic!("{} is out of range on ValueArray {}", index, self.name);
        }
        &self.values[index]
    }

    pub fn write(&mut self, value: SquatValue) {
        self.values.push(value);
    }
}
