use std::{
    fmt::{Display, Formatter},
    num::ParseFloatError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    code: ErrorCode,
    position: Option<Position>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorCode {
    Message(String),

    EofParsingBool,
    EofParsingComment,
    EofParsingNull,
    EofParsingNumber,
    EofParsingString,
    EofParsingValue,

    ExpectedBool,
    ExpectedComment,
    ExpectedNull,
    ExpectedNumber,
    ExpectedString,

    InvalidEscapeSequence,
    LeadingZero,
    LineTerminatorInString,
    OverflowParsingNumber,
    TrailingCharacters,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ErrorCode::Message(msg) => write!(f, "{msg}"),

            ErrorCode::EofParsingBool => write!(f, "EOF parsing bool"),
            ErrorCode::EofParsingComment => write!(f, "EOF parsing comment"),
            ErrorCode::EofParsingNull => write!(f, "EOF parsing null"),
            ErrorCode::EofParsingNumber => write!(f, "EOF parsing number"),
            ErrorCode::EofParsingString => write!(f, "EOF parsing string"),
            ErrorCode::EofParsingValue => write!(f, "EOF parsing value"),

            ErrorCode::ExpectedBool => write!(f, "expected bool"),
            ErrorCode::ExpectedComment => write!(f, "expected comment"),
            ErrorCode::ExpectedNull => write!(f, "expected null"),
            ErrorCode::ExpectedNumber => write!(f, "expected number"),
            ErrorCode::ExpectedString => write!(f, "expected string"),

            ErrorCode::InvalidEscapeSequence => write!(f, "invalid escape sequence"),
            ErrorCode::LeadingZero => write!(f, "leading zero"),
            ErrorCode::LineTerminatorInString => write!(f, "line terminator in string"),
            ErrorCode::OverflowParsingNumber => write!(f, "overflow parsing number"),
            ErrorCode::TrailingCharacters => write!(f, "trailing characters"),
        }
    }
}

impl Error {
    #[must_use]
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            position: None,
        }
    }

    #[must_use]
    pub fn new_at(position: Position, code: ErrorCode) -> Self {
        Self {
            code,
            position: Some(position),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new(ErrorCode::Message(msg.to_string()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some(position) = self.position {
            write!(f, "{} at {}", self.code, position)
        } else {
            write!(f, "{}", self.code)
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position {
    // The first line is line 0
    pub line: usize,
    // The first column is column 0
    pub column: usize,
}

impl Position {
    #[must_use]
    pub fn from_offset(offset: usize, input: &str) -> Self {
        let mut res = Self { line: 0, column: 0 };
        let mut chars = input[..offset].chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                json5_line_terminator!() => {
                    // "The character sequence <CR><LF> is commonly used as a line terminator. It
                    // should be considered a single character for the purpose of reporting line
                    // numbers." – https://262.ecma-international.org/5.1/#sec-7.3
                    if c == '\u{000D}' && chars.peek() == Some(&'\u{000A}') {
                        chars.next();
                    }
                    res.line += 1;
                    res.column = 0;
                }
                _ => res.column += 1,
            }
        }
        res
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "line {} column {}", self.line + 1, self.column + 1)
    }
}
