use pest::Span;
use serde::{de, ser};
use std::fmt::{self, Display};

use crate::de::Rule;

/// Alias for a `Result` with error type `json5::Error`
pub type Result<T> = std::result::Result<T, Error>;

/// One-based line and column at which the error was detected.
#[derive(Clone, Debug, PartialEq)]
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

/// A bare bones error type which currently just collapses all the underlying errors in to a single
/// string... This is fine for displaying to the user, but not very useful otherwise. Work to be
/// done here.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Just shove everything in a single variant for now.
    Message {
        /// The error message.
        msg: String,
        /// The location of the error, if applicable.
        location: Option<Location>,
    },
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(err: pest::error::Error<Rule>) -> Self {
        let (line, column) = match err.line_col {
            pest::error::LineColLocation::Pos((l, c)) => (l, c),
            pest::error::LineColLocation::Span((l, c), (_, _)) => (l, c),
        };

        let msg = match err.variant {
            pest::error::ErrorVariant::ParsingError {
                positives,
                negatives,
            } => match (negatives.is_empty(), positives.is_empty()) {
                (false, false) => format!(
                    "unexpected {}; expected {}",
                    Enumerated(&negatives),
                    Enumerated(&positives),
                ),
                (false, true) => format!("unexpected {}", Enumerated(&negatives)),
                (true, false) => format!("expected {}", Enumerated(&positives)),
                (true, true) => "unknown parsing error".to_owned(),
            },
            pest::error::ErrorVariant::CustomError { message } => message,
        };

        Error::Message {
            msg,
            location: Some(Location { line, column }),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message {
            msg: msg.to_string(),
            location: None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message {
            msg: msg.to_string(),
            location: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message { ref msg, .. } => write!(formatter, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Adds location information from `span`, if `res` is an error.
pub fn set_location<T>(res: &mut Result<T>, span: &Span<'_>) {
    if let Err(ref mut e) = res {
        let Error::Message { location, .. } = e;
        if location.is_none() {
            let (line, column) = span.start_pos().line_col();
            *location = Some(Location { line, column });
        }
    }
}

struct Enumerated<'a, R: pest::RuleType>(&'a [R]);

impl<R: pest::RuleType> Display for Enumerated<'_, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            [] => Ok(()),
            [rule] => write!(f, "{:?}", rule),
            [first, second] => write!(f, "{:?} or {:?}", first, second),
            rules => {
                let mut iter = rules.iter();
                let last = iter.next_back().expect("last");
                for rule in iter {
                    write!(f, "{:?}, ", rule)?;
                }
                write!(f, "or {:?}", last)
            }
        }
    }
}
