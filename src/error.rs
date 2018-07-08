use pest;
use serde::{de, ser};
use std;
use std::fmt::{self, Display};

use de::Rule;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    Parse(pest::error::Error<Rule>),
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(err: pest::error::Error<Rule>) -> Self {
        Error::Parse(err)
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

// TODO this is "soft depricated", what does that mean?
impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::Parse(_) => "parsing error!", // TODO display error!
        }
    }
}
