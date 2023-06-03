use crate::value::ValueArray;
use crate::op_code::OpCode;

#[derive(Debug, PartialEq)]
struct Line {
    line: u32,
    count: u32
}

impl Line {
    fn new(line: u32) -> Line {
        Line {
            line,
            count: 1
        }
    }

    fn increment(&mut self) {
        self.count += 1;
    }
}

#[derive(Debug)]
pub struct Chunk {
    name: String,
    code: Vec<OpCode>,
    lines: Vec<Line>,
    constants: ValueArray
}

impl Chunk {
    pub fn new(name: String) -> Chunk {
        Chunk {
            name: String::from(&name) + " chunk",
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(name + " constants")
        }
    }

    pub fn add_constant(&mut self, value: f64) -> usize {
        self.constants.write(value);
        self.constants.count() - 1
    }

    pub fn disassemble(&self) {
        println!("-- {} --", self.name);

        let mut op_index: usize = 0;
        let mut prev_line: u32 = 0;
        while op_index < self.code.len() {
            let op_code = &self.code[op_index];
            // If this lines panics, there is something wrong with the implementation
            let line = self.get_line(op_index).unwrap();

            let identifier: String;
            if op_index != 0 && line == prev_line {
                identifier = format!("{:04} {:04}", op_index, "    ");
            } else {
                identifier = format!("{:04} {:04}", op_index, line);
            }
            prev_line = line;

            op_index = self.dismantle_op_code(identifier, &op_code, op_index);
        }
    }

    fn dismantle_op_code(&self, identifier: String, op_code: &OpCode, op_index: usize) -> usize  {
        match op_code {
            OpCode::Constant => {
                if op_index == self.code.len() - 1 {
                    panic!("Constant OpCode must be followed by Index - {}", identifier)
                }
                else if let OpCode::Index(index) = self.code[op_index + 1] {
                    let value: f64 = self.constants.get(index);
                    println!("{}: {:?} {:?} {:?}", identifier, op_code, &self.code[op_index + 1], value);
                    op_index + 2
                } else {
                    panic!("Constant OpCode must be followed by Index - {}", identifier)
                }
            },
            OpCode::Return => {
                println!("{}: {:?}", identifier, op_code);
                op_index + 1
            },
            _ => panic!("Unsupported OpCode - {} {:?}", identifier, op_code)
        }
    }

    pub fn free(&mut self) {
        self.code.clear();
        self.lines.clear();
        self.constants.free();
    }

    fn get_line(&self, op_index: usize) -> Option<u32> {
        let mut i = 0;
        for it in self.lines.iter() {
            for _j in 0..it.count {
                if i == op_index {
                    return Some(it.line);
                }
                i += 1;
            }
        }
        
        None
    }

    pub fn write(&mut self, byte: OpCode, line: u32) {
        self.code.push(byte);
        if let Some(last) = self.lines.last_mut() {
            if last.line == line {
                last.increment();
                return;
            }
        }
        self.lines.push(Line::new(line));
    }
}
