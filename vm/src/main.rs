mod chunk;
mod compiler;
mod lexer;
mod native;
mod object;
mod op_code;
mod options;
mod token;
mod value;
mod vm;

use options::Options;
use std::fs;
use vm::{InterpretResult, VM};

fn run_file(opts: &Options) -> Result<i64, i64> {
    let mut vm = VM::new();

    let source = match fs::read_to_string(&opts.file) {
        Ok(contents) => contents,
        _ => panic!("Failed to read file '{}'", opts.file),
    };
    println!("Compiling and running file: {}", opts.file);

    let result = vm.interpret_source(source, opts);

    match result {
        InterpretResult::InterpretOk(exit_code) => Ok(exit_code),
        InterpretResult::InterpretCompileError => Err(-1),
        InterpretResult::InterpretRuntimeError => Err(-1),
    }
}

fn cmain() -> i32 {
    env_logger::init();
    let opts = Options::parse();

    match run_file(&opts) {
        Ok(i) | Err(i) => i as i32,
    }
}

fn main() {
    std::process::exit(cmain());
}
