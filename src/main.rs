mod chunk;
mod op_code;
mod value;

use std::env;
use chunk::Chunk;
use op_code::OpCode;

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    let mut chunk = Chunk::new("test".to_owned());
    let constant_index = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant, 0);
    chunk.write(OpCode::Index(constant_index), 0);
    chunk.write(OpCode::Return, 0);
    chunk.disassemble();
    chunk.free();

    Ok(())
}
