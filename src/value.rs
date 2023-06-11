#[derive(Debug, Clone)]
pub enum SquatValue {
    Nil,
    F64(f64),
    String(String),
    Bool(bool)
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
