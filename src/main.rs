mod chunk;
mod compiler;
mod lexer;
mod op_code;
mod token;
mod value;
mod vm;

use std::{env, fs, io::{self, Write}};
use log::{debug, error, info};
use vm::{VM, InterpretResult};

fn repl() {
    info!("Starting repl");

    let mut vm = VM::new();
    let stdin = io::stdin();

    loop {
        print!("> ");
        if let Err(err) = io::stdout().flush() {
            error!("Failed to flush: {}", err);
            break;
        }

        let mut user_input = String::new();
        if let Err(err) = stdin.read_line(&mut user_input) {
            error!("Failed to read user input: {}", err);
            continue;
        }

        if user_input.trim().len() == 0 {
            break;
        }

        vm.interpret_source(user_input);
    }
}

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

    if args.len() == 1 {
        panic!("Repl support went caput with the adition of the main function as entry point");
        repl();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        error!("Usage: squat [path]\n");
        return Err(());
    }

    Ok(())
}
