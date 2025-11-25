use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    inner: Box<ErrorInner>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ErrorInner {
    content: ErrorContent,
    position: Option<Position>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorContent {
    Code(ErrorCode),
    Custom(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorCode {
    EofParsingArray,
    EofParsingBool,
    EofParsingComment,
    EofParsingEscapeSequence,
    EofParsingIdentifier,
    EofParsingNull,
    EofParsingNumber,
    EofParsingObject,
    EofParsingString,
    EofParsingValue,

    ExpectedBool,
    ExpectedClosingBrace,
    ExpectedClosingBracket,
    ExpectedColon,
    ExpectedComma,
    ExpectedComment,
    ExpectedIdentifier,
    ExpectedNull,
    ExpectedNumber,
    ExpectedOpeningBrace,
    ExpectedOpeningBracket,
    ExpectedString,
    ExpectedStringOrObject,
    ExpectedValue,

    InvalidBytes,
    InvalidEscapeSequence,
    LeadingZero,
    LineTerminatorInString,
    OverflowParsingNumber,
    TrailingCharacters,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ErrorCode::EofParsingArray => write!(f, "EOF parsing array"),
            ErrorCode::EofParsingBool => write!(f, "EOF parsing bool"),
            ErrorCode::EofParsingComment => write!(f, "EOF parsing comment"),
            ErrorCode::EofParsingEscapeSequence => write!(f, "EOF parsing escape sequence"),
            ErrorCode::EofParsingIdentifier => write!(f, "EOF parsing identifier"),
            ErrorCode::EofParsingNull => write!(f, "EOF parsing null"),
            ErrorCode::EofParsingNumber => write!(f, "EOF parsing number"),
            ErrorCode::EofParsingObject => write!(f, "EOF parsing object"),
            ErrorCode::EofParsingString => write!(f, "EOF parsing string"),
            ErrorCode::EofParsingValue => write!(f, "EOF parsing value"),

            ErrorCode::ExpectedBool => write!(f, "expected bool"),
            ErrorCode::ExpectedClosingBrace => write!(f, "expected closing brace"),
            ErrorCode::ExpectedClosingBracket => write!(f, "expected closing bracket"),
            ErrorCode::ExpectedColon => write!(f, "expected colon"),
            ErrorCode::ExpectedComma => write!(f, "expected comma"),
            ErrorCode::ExpectedComment => write!(f, "expected comment"),
            ErrorCode::ExpectedIdentifier => write!(f, "expected identifier"),
            ErrorCode::ExpectedNull => write!(f, "expected null"),
            ErrorCode::ExpectedNumber => write!(f, "expected number"),
            ErrorCode::ExpectedOpeningBrace => write!(f, "expected opening brace"),
            ErrorCode::ExpectedOpeningBracket => write!(f, "expected opening bracket"),
            ErrorCode::ExpectedString => write!(f, "expected string"),
            ErrorCode::ExpectedStringOrObject => write!(f, "expected string or object"),
            ErrorCode::ExpectedValue => write!(f, "expected value"),

            ErrorCode::InvalidBytes => write!(f, "invalid bytes"),
            ErrorCode::InvalidEscapeSequence => write!(f, "invalid escape sequence"),
            ErrorCode::LeadingZero => write!(f, "leading zero"),
            ErrorCode::LineTerminatorInString => write!(f, "line terminator in string"),
            ErrorCode::OverflowParsingNumber => write!(f, "overflow parsing number"),
            ErrorCode::TrailingCharacters => write!(f, "trailing characters"),
        }
    }
}

impl Display for ErrorContent {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ErrorContent::Code(code) => write!(f, "{code}"),
            ErrorContent::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some(position) = self.inner.position {
            write!(f, "{} at {}", self.inner.content, position)
        } else {
            write!(f, "{}", self.inner.content)
        }
    }
}

impl Error {
    #[must_use]
    pub fn new(code: ErrorCode) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                content: ErrorContent::Code(code),
                position: None,
            }),
        }
    }

    #[must_use]
    pub fn new_at(position: Position, code: ErrorCode) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                content: ErrorContent::Code(code),
                position: Some(position),
            }),
        }
    }

    #[must_use]
    pub fn custom<T: Display>(msg: T) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                content: ErrorContent::Custom(msg.to_string()),
                position: None,
            }),
        }
    }

    #[must_use]
    pub fn custom_at<T: Display>(position: Position, msg: T) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                content: ErrorContent::Custom(msg.to_string()),
                position: Some(position),
            }),
        }
    }

    #[must_use]
    pub fn with_position(mut self, position: Position) -> Self {
        if self.inner.position.is_none() {
            self.inner.position = Some(position);
        }
        self
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::custom(msg)
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::custom(msg)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::custom(err)
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
            if crate::char::is_json5_line_terminator(c) {
                // "The character sequence <CR><LF> is commonly used as a line terminator. It
                // should be considered a single character for the purpose of reporting line
                // numbers." – https://262.ecma-international.org/5.1/#sec-7.3
                if c == '\u{000D}' && chars.peek() == Some(&'\u{000A}') {
                    chars.next();
                }
                res.line += 1;
                res.column = 0;
            } else {
                res.column += 1;
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
