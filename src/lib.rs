extern crate pest;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate pest_derive;

mod de;
mod error;

pub use de::{from_str, Deserializer};
pub use error::{Error, Result};
