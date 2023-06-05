mod chunk;
mod op_code;
mod value;
mod vm;

use std::env;
use chunk::Chunk;
use op_code::OpCode;
use vm::VM;

fn main() -> Result<(), ()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    dbg!(args);

    let mut chunk = Chunk::new("test".to_owned());
    // Calculate ((1 + 3) * 10 - 2) / 2
    let constant1 = chunk.add_constant(1.);
    chunk.write(OpCode::Constant, 0);
    chunk.write(OpCode::Index(constant1), 0);

    let constant2 = chunk.add_constant(3.);
    chunk.write(OpCode::Constant, 0);
    chunk.write(OpCode::Index(constant2), 0);

    chunk.write(OpCode::Add, 0);

    let constant3 = chunk.add_constant(10.);
    chunk.write(OpCode::Constant, 0);
    chunk.write(OpCode::Index(constant3), 0);

    chunk.write(OpCode::Multiply, 0);

    let constant4 = chunk.add_constant(2.);
    chunk.write(OpCode::Constant, 0);
    chunk.write(OpCode::Index(constant4), 0);
    
    chunk.write(OpCode::Subtract, 0);

    chunk.write(OpCode::Constant, 0);
    chunk.write(OpCode::Index(constant4), 0);
    
    chunk.write(OpCode::Divide, 0);

    chunk.write(OpCode::Negate, 0);
    chunk.write(OpCode::Return, 0);
    chunk.disassemble();

    let mut vm = VM::new(chunk);

    vm.interpret();

    Ok(())
}
