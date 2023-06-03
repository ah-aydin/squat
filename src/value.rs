#[derive(Debug)]
pub struct ValueArray {
    name: String,
    values: Vec<f64>,
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

    pub fn free(&mut self) {
        self.values.clear();
    }

    pub fn get(&self, index: usize) -> f64 {
        if index >= self.values.len() {
            panic!("{} is out of range on ValueArray {}", index, self.name);
        }
        self.values[index]
    }

    pub fn write(&mut self, value: f64) {
        self.values.push(value);
    }
}
