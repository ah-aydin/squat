use crate::value::ValueArray;
use crate::op_code::OpCode;

use log::debug;

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
    pub current_instruction: usize,
    lines: Vec<Line>,
    constants: ValueArray
}

impl Chunk {
    pub fn new(name: String) -> Chunk {
        Chunk {
            name: String::from(&name) + " chunk",
            code: Vec::new(),
            current_instruction: 0,
            lines: Vec::new(),
            constants: ValueArray::new(name + " constants")
        }
    }

    pub fn add_constant(&mut self, value: f64) -> usize {
        self.constants.write(value);
        self.constants.count() - 1
    }

    pub fn read_constant(&self, index: usize) -> f64 {
        self.constants.get(index)
    }

    pub fn disassemble(&self) {
        debug!("==== {} ====", self.name);

        let mut op_index: usize = 0;
        while op_index < self.code.len() {
            let op_code = &self.code[op_index];
            op_index = self.disassemble_instruction(op_code, op_index);
        }
    }

    pub fn disassemble_current_instruction(&self) {
        let op_code = &self.code[self.current_instruction];
        self.disassemble_instruction(op_code, self.current_instruction);
    }

    fn disassemble_instruction(&self, op_code: &OpCode, op_index: usize) -> usize  {
        // If this lines panics, there is something wrong with the implementation
        let identifier = format!("{:04} {:04}", op_index, self.get_line(op_index).unwrap());

        match op_code {
            OpCode::Constant => {
                if op_index == self.code.len() - 1 {
                    panic!("Constant OpCode must be followed by Index - {}", identifier)
                }
                else if let OpCode::Index(index) = self.code[op_index + 1] {
                    let value: f64 = self.constants.get(index);
                    debug!("{}: {:?} {:?} {:?}", identifier, op_code, &self.code[op_index + 1], value);
                    op_index + 2
                } else {
                    panic!("Constant OpCode must be followed by Index - {}", identifier)
                }
            },
            _ => {
                debug!("{}: {:?}", identifier, op_code);
                op_index + 1
            }
        }
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

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn next(&mut self) -> Option<&OpCode> {
        if self.current_instruction < self.code.len() {
            let next_op = &self.code[self.current_instruction];
            self.current_instruction += 1;
            return Some(next_op);
        }
        None
    }

    pub fn reset(&mut self) {
        self.current_instruction = 0;
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
