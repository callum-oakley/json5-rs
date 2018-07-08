extern crate pest;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate pest_derive;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

mod de;
mod error;

pub use de::{from_str, Json5Deserializer as Deserializer};
pub use error::{Error, Result};
