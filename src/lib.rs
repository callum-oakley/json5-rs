//! [JSON5][] is a superset of [JSON][] with an expanded syntax including some productions from
//! [ECMAScript 5.1][]. It aims to be easier to write and maintain by hand (e.g. for config files).
//! It is not intended to be used for machine-to-machine communication, for which you'd be better
//! served by [serde-rs/json][].
//!
//! In particular, JSON5 allows comments, trailing commas, object keys without quotes, single quoted
//! strings, hexadecimal numbers, multi-line strings...
//!
//! ```json5
//! {
//!   // comments
//!   unquoted: 'and you can quote me on that',
//!   singleQuotes: 'I can use "double quotes" here',
//!   lineBreaks: "Look, Mom! \
//! No \\n's!",
//!   hexadecimal: 0xdecaf,
//!   leadingDecimalPoint: .8675309, andTrailing: 8675309.,
//!   positiveSign: +1,
//!   trailingComma: 'in objects', andIn: ['arrays',],
//!   "backwardsCompatible": "with JSON",
//! }
//! ```
//!
//! This crate provides functions for deserializing JSON5 text into a Rust datatype and for
//! serializing a Rust datatype as JSON5 text, both via the [Serde framework][].
//!
//! # Deserialization
//!
//! Implementing [`serde::Deserialize`] on your type will allow you to parse JSON5 text into a value
//! of that type with [`from_str`].
//!
//! ```
//! use serde_derive::Deserialize;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Config<'a> {
//!     foo: u32,
//!     bar: &'a str,
//! }
//!
//! let config: Config = json5::from_str("
//!   {
//!     // Note unquoted keys, comments, and trailing commas.
//!     foo: 42,
//!     bar: 'baz',
//!   }
//! ")?;
//!
//! assert_eq!(config, Config{ foo: 42, bar: "baz" });
//! # Ok::<(), json5::Error>(())
//! ```
//!
//! There are many ways to customize the deserialization (e.g. deserializing `camelCase` field names
//! into a struct with `snake_case` fields). See the Serde docs, especially the [Attributes][],
//! [Custom serialization][], and [Examples][] sections.
//!
//! ## Configuration
//!
//! The [`DeserializerOptions`] struct allows you to adjust parsing behavior. For example, you can
//! allow line terminators (newlines) inside string literals, which is normally a syntax error:
//!
//! ```
//! use json5::{Deserializer, DeserializerOptions};
//!
//! let options = DeserializerOptions {
//!     allow_line_terminators_in_strings: true,
//!     strip_line_terminators_from_keys: false,
//! };
//! let mut deserializer = Deserializer::from_str_with_options(r#"'multi
//! line
//! string'"#, options);
//! let s: String = serde::Deserialize::deserialize(&mut deserializer).unwrap();
//! assert_eq!(s, "multi\nline\nstring");
//! ```
//!
//! # Serialization
//!
//! Similarly, implementing [`serde::Serialize`] on a Rust type allows you to produce a JSON5
//! serialization of values of that type with [`to_string`] or [`to_writer`]. The serializer will
//! omit quotes around object keys where possible and will indent nested objects and arrays, but is
//! otherwise fairly basic.
//!
//! ```
//! use serde_derive::Serialize;
//!
//! #[derive(Serialize)]
//! struct Config<'a> {
//!     foo: u32,
//!     bar: &'a str,
//! }
//!
//! let config = Config {
//!     foo: 42,
//!     bar: "baz",
//! };
//!
//! assert_eq!(&json5::to_string(&config)?, "{
//!   foo: 42,
//!   bar: \"baz\",
//! }");
//! # Ok::<(), json5::Error>(())
//! ```
//!
//! There are many ways to customize the serialization (e.g. serializing `snake_case` struct fields
//! as `camelCase`). See the Serde docs, especially the [Attributes][], [Custom serialization][] and
//! [Examples][] sections.
//!
//! # Byte arrays
//!
//! All the types of the [Serde data model][] are supported. Byte arrays are encoded as hex strings.
//! e.g.
//!
//! ```
//! use serde_bytes::{Bytes, ByteBuf};
//!
//! let s = json5::to_string(&Bytes::new(b"JSON5"))?;
//! assert_eq!(&s, "\"4a534f4e35\"");
//! assert_eq!(json5::from_str::<ByteBuf>(&s)?, ByteBuf::from("JSON5"));
//! # Ok::<(), json5::Error>(())
//! ```
//!
//! [Attributes]: https://serde.rs/attributes.html
//! [Custom serialization]: https://serde.rs/custom-serialization.html
//! [ECMAScript 5.1]: https://www.ecma-international.org/ecma-262/5.1/
//! [Examples]: https://serde.rs/examples.html
//! [JSON]: https://www.json.org/json-en.html
//! [JSON5]: https://json5.org/
//! [Serde data model]: https://serde.rs/data-model.html#types
//! [Serde framework]: https://serde.rs/
//! [serde-rs/json]: https://github.com/serde-rs/json

#![warn(clippy::pedantic)]

#[macro_use]
mod de;
mod char;
mod error;
mod ser;

#[allow(clippy::all, clippy::pedantic, dead_code)]
mod unicode;

pub use de::{Deserializer, DeserializerOptions, from_str, from_str_with_options};
pub use error::{Error, ErrorCode, Position};
pub use ser::{Serializer, to_string, to_writer};
