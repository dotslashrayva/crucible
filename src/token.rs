#[derive(Debug, PartialEq)]
pub enum Token {
    Int,
    Void,
    Return,

    Identifier(String),
    Constant(String),

    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,

    Tilde,
    Exclaim,
    PlusPlus,
    MinusMinus,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,

    Pipe,
    Caret,
    Ampersand,
    LessLess,
    GreaterGreater,

    AmpAmp,
    PipePipe,

    Less,
    LessEqual,

    Greater,
    GreaterEqual,

    EqualEqual,
    ExclaimEqual,

    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,

    AmpEqual,
    PipeEqual,
    CaretEqual,
    LessLessEqual,
    GreaterGreaterEqual,

    EOF,
}
