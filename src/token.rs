#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
pub enum TokenType {
    // Single-character tokens
    LeftParenthesis, RightParenthesis, LeftBrace, RightBrace, LeftBracket, RightBracket,
    Comma, Dot, Minus, Semicolon, Slash, Star, Colon, Question,

    // One or two character tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    Plus, PlusPlus,

    // Literals
    Identifier, String, Number,
    
    // Keywords
    And, Break, Class, Else, Extends, False, For, Func, If, Nil, Or, Print,
    Return, Static, Super, This, True, Var, While,

    Comment, Eof
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: u32
}
