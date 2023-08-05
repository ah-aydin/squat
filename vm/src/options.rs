use arg_parser::CmdArgs;

#[derive(CmdArgs, Debug, Default)]
#[metadata(description = "Squat virtual machine.")]
pub struct Options {
    #[arg(
        short = "-f",
        long = "--file",
        description = "The file to compile",
        required = true
    )]
    pub file: String,

    #[arg(
        short = "-c",
        long = "--code",
        description = "Log byte code after compilation"
    )]
    pub log_byte_code: bool,

    #[arg(short = "-g", long = "--globals", description = "Log global variables")]
    pub log_globals: bool,

    #[arg(
        short = "-i",
        long = "--instructions",
        description = "Log each instruction before execution"
    )]
    pub log_insturctions: bool,

    #[arg(
        short = "-s",
        long = "--stack",
        description = "Log the stack of the program before each instruction"
    )]
    pub log_stack: bool,
}
