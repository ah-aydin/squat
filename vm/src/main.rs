mod args;
mod chunk;
mod compiler;
mod lexer;
mod op_code;
mod token;
mod value;
mod vm;

use std::fs;
use args::Options;
use vm::{VM, InterpretResult};


fn run_file(opts: &Options) {
    let mut vm = VM::new();

    let source = match fs::read_to_string(&opts.file) {
        Ok(contents) => contents,
        _ => panic!("Failed to read file '{}'", opts.file)
    };
    println!("Compiling and running file: {}", opts.file);

    let result = vm.interpret_source(source, opts);

    if result == InterpretResult::InterpretCompileError {
        println!("CompileError");
    }
    if result == InterpretResult::InterpretRuntimeError {
        println!("RuntimeError");
    }
}

fn main() -> Result<(), ()> {
    env_logger::init();
    let opts = Options::parse();

    run_file(&opts);

    Ok(())
}
