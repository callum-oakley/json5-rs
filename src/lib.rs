#![warn(clippy::pedantic)]

#[macro_use]
mod de;
mod char;
mod error;
mod ser;

#[allow(clippy::all, clippy::pedantic, dead_code)]
mod unicode;

pub use de::{Deserializer, from_str};
pub use error::{Error, ErrorCode, Position};
pub use ser::{Serializer, to_string};
