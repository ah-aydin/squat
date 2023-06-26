mod chunk;
mod compiler;
mod lexer;
mod op_code;
mod token;
mod value;
mod vm;

use std::fs;
use vm::{VM, InterpretResult};

fn run_file(file: &String) {
    let mut vm = VM::new();

    let source = match fs::read_to_string(&file) {
        Ok(contents) => contents,
        _ => panic!("Failed to read file '{}'", file)
    };
    println!("Compiling and running file: {}", file);

    let result = vm.interpret_source(source);

    if result == InterpretResult::InterpretCompileError {
        println!("CompileError");
    }
    if result == InterpretResult::InterpretRuntimeError {
        println!("RuntimeError");
    }
}

use arg_parser::CmdArgs;

#[derive(CmdArgs, Debug, Default)]
struct Args {
    #[arg(short="-f", long="--file", description="The file to compile", required=true)]
    file: String,

    #[arg(short="-c", long="--code", description="Log byte code after compilation")]
    log_byte_code: bool,

    #[arg(short="-g", long="--globals", description="Log global variable indicies")]
    log_globals: bool,

    #[arg(short="-i", long="--instructions", description="Log each instruction before execution")]
    log_insturctions: bool,

    #[arg(short="-s", long="--stack", description="Log the stack of the program before each instruction")]
    log_stack: bool,
}

fn main() -> Result<(), ()> {
    env_logger::init();

    let args = Args::parse();
    println!("{:?}", args);

    // let args: Vec<String> = env::args().collect();
    // debug!("{:?}", args);

    run_file(&args.file);

    Ok(())
}
