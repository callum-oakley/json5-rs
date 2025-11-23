#![warn(clippy::pedantic)]

#[macro_use]
mod de;
mod error;

#[allow(clippy::all, clippy::pedantic, dead_code)]
mod unicode;

pub use {
    de::{Deserializer, from_str},
    error::{Error, ErrorCode, Position},
};
