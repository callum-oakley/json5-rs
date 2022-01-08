use pest::Span;
use serde::{de, ser};
use std::fmt::{self, Display};

use crate::de::Rule;

/// Alias for a `Result` with error type `json5::Error`
pub type Result<T> = std::result::Result<T, Error>;

/// One-based line and column at which the error was detected.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    /// The one-based line number of the error.
    pub line: usize,
    /// The one-based column number of the error.
    pub column: usize,
}

impl From<&Span<'_>> for Location {
    fn from(s: &Span<'_>) -> Self {
        let (line, column) = s.start_pos().line_col();
        Self { line, column }
    }
}

/// An error during serialization or deserialization of JSON5.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    // TODO: Just stringify all errors for now.
    msg: String,
    location: Option<Location>,
}

impl Error {
    /// Returns the location that the error occurred, if applicable.
    #[must_use]
    pub fn location(&self) -> Option<Location> {
        self.location
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(err: pest::error::Error<Rule>) -> Self {
        let (line, column) = match err.line_col {
            pest::error::LineColLocation::Pos((l, c)) => (l, c),
            pest::error::LineColLocation::Span((l, c), (_, _)) => (l, c),
        };
        Error {
            msg: err.to_string(),
            location: Some(Location { line, column }),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error {
            msg: msg.to_string(),
            location: None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error {
            msg: msg.to_string(),
            location: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(formatter)
    }
}

impl std::error::Error for Error {}

/// Adds location information from `span`, if `res` is an error.
pub fn set_location<T>(res: &mut Result<T>, span: &Span<'_>) {
    if let Err(ref mut e) = res {
        e.location.get_or_insert_with(|| {
            let (line, column) = span.start_pos().line_col();
            Location { line, column }
        });
    }
}
