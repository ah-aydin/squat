pub mod squat_type;
pub mod squat_value;

use crate::object::SquatObject;
use squat_value::SquatValue;

#[derive(Debug)]
pub struct ValueArray {
    name: String,
    values: Vec<SquatValue>,
}

impl ValueArray {
    pub fn new(name: &str) -> ValueArray {
        ValueArray {
            name: String::from(name),
            values: Vec::new(),
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
