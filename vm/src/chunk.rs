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
}

impl Chunk {
    pub fn new(name: &str) -> Chunk {
        Chunk {
            name: String::from(name) + " Chunk",
            code: Vec::new(),
            current_instruction: 0,
            lines: Vec::new(),
        }
    }

    #[cfg(debug_assertions)]
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

    pub fn get_current_instruction_line(&self) -> u32 {
        self.get_line(self.current_instruction).unwrap_or(0)
    }

    fn disassemble_instruction(&self, op_code: &OpCode, op_index: usize) -> usize  {
        // If this lines panics, there is something wrong with the implementation
        let identifier = format!("{:08} {:08}", op_index, self.get_line(op_index).unwrap());

        match op_code {
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

    pub fn set_jump_at(&mut self, location: usize, offset: usize) {
        match self.code[location] {
            OpCode::JumpIfFalse(_) => self.code[location] = OpCode::JumpIfFalse(offset),
            OpCode::Jump(_) => self.code[location] = OpCode::Jump(offset),
            OpCode::JumpIfTrue(_) => self.code[location] = OpCode::JumpIfTrue(offset),
            _ => panic!("Trying to modify instruction {:?} into a jump instruction", self.code[location])
        };
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_size(&self) -> usize {
        self.code.len()
    }

    pub fn next(&mut self) -> Option<&OpCode> {
        if self.current_instruction < self.code.len() {
            let next_op = &self.code[self.current_instruction];
            self.current_instruction += 1;
            return Some(next_op);
        }
        None
    }

    pub fn write(&mut self, op_code: OpCode, line: u32) {
        self.code.push(op_code);
        if let Some(last) = self.lines.last_mut() {
            if last.line == line {
                last.increment();
                return;
            }
        }
        self.lines.push(Line::new(line));
    }

    pub fn clear_instructions(&mut self) {
        self.code.clear();
    }
}
