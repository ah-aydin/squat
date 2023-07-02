mod chunk;
mod compiler;
mod lexer;
mod object;
mod op_code;
mod options;
mod token;
mod value;
mod vm;

use std::fs;
use options::Options;
use vm::{VM, InterpretResult};

use crate::value::SquatValue;


fn run_file(opts: &Options) -> Result<(),()> {
    let mut vm = VM::new();

    let source = match fs::read_to_string(&opts.file) {
        Ok(contents) => contents,
        _ => panic!("Failed to read file '{}'", opts.file)
    };
    println!("Compiling and running file: {}", opts.file);

    let result = vm.interpret_source(source, opts);

    if result == InterpretResult::InterpretCompileError {
        println!("CompileError");
        return Err(());
    } else if result == InterpretResult::InterpretRuntimeError {
        println!("RuntimeError");
        return Err(());
    } else if let InterpretResult::InterpretOk(value) = result {
        let exit_code = match value {
            SquatValue::Nil => 0.,
            SquatValue::Number(value) => value,
            _ => {
                println!("[ERROR] Function 'main' can only return numbers");
                return Ok(());
            }
        };

        println!("Exit code: {exit_code}");
    }
    
    return Ok(());
}

fn main() -> Result<(), ()> {
    env_logger::init();
    let opts = Options::parse();

    run_file(&opts)
}
