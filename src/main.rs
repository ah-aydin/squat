mod chunk;
mod compiler;
mod lexer;
mod op_code;
mod token;
mod value;
mod vm;

use std::{env, fs};
use log::{debug, error, info};
use vm::{VM, InterpretResult};

fn run_file(file: &String) {
    let mut vm = VM::new();

    info!("Reading file: {}", file);
    let source = match fs::read_to_string(&file) {
        Ok(contents) => contents,
        _ => panic!("Failed to read file '{}'", file)
    };

    let result = vm.interpret_source(source);

    if result == InterpretResult::InterpretCompileError {
        println!("CompileError");
    }
    if result == InterpretResult::InterpretRuntimeError {
        println!("RuntimeError");
    }
}

fn main() -> Result<(), ()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    debug!("{:?}", args);

    if args.len() == 2 {
        run_file(&args[1]);
    } else {
        error!("Usage: squat [path]\n");
        return Err(());
    }

    Ok(())
}
