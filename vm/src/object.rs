#[derive(Debug, Clone)]
pub struct SquatFunction {
    pub name: String,
    pub start_instruction_index: usize,
    pub arity: usize
}

impl SquatFunction {
    pub fn new(name: &str, start_instruction_index: usize, arity: usize) -> SquatFunction {
        SquatFunction {
            name: name.to_owned(),
            start_instruction_index,
            arity
        }
    }
}

#[derive(Debug, Clone)]
pub enum SquatObject {
    Function(SquatFunction)
}

impl ToString for SquatObject {
    fn to_string(&self) -> String {
        match self {
            SquatObject::Function(func) => format!("<fn {}>", func.name)
        }
    }
}

impl PartialEq for SquatObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                SquatObject::Function(func1), 
                SquatObject::Function(func2)
            ) => func1.start_instruction_index == func2.start_instruction_index
        }
    }
}
