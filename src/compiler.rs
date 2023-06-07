use crate::lexer::{TokenType, Lexer};

use log::error;

pub fn compile(source: String) {
    let mut lexer = Lexer::new(&source);
    // TODO ("This is just to test the lexer, remove this after compiler comes in")
    let mut line: u32 = 0;
    loop {
        let token = lexer.scan_token();
        match token {
            Ok(token) => {
                if token.token_type == TokenType::Eof {
                    break;
                }
                if token.line != line {
                    print!("{:>4} | ", token.line);
                    line = token.line;
                } else {
                    print!("     | ");
                }
                println!("{:?}", token);
            },
            Err(err) => {
                error!("{:?}", err);
            }
        }
    }
}
