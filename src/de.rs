use std::{
    fmt::Display,
    iter::Peekable,
    ops::Deref,
    str::{CharIndices, FromStr},
};

use serde::{Deserialize, de::Visitor, forward_to_deserialize_any};

use crate::error::{Error, ErrorCode, Position, Result};

/// Parse a JSON5 string and map it to a type implementing [`Deserialize`].
///
/// # Example
/// ```
/// use serde_derive::Deserialize;
///
/// #[derive(Debug, PartialEq, Deserialize)]
/// struct Config<'a> {
///     foo: u32,
///     bar: &'a str,
/// }
///
/// let config: Config = json5::from_str("
///   {
///     // Note unquoted keys, comments, and trailing commas.
///     foo: 42,
///     bar: 'baz',
///   }
/// ")?;
///
/// assert_eq!(config, Config{ foo: 42, bar: "baz" });
/// # Ok::<(), json5::Error>(())
/// ```
///
/// # Errors
/// Fails if the JSON5 is malformed or we can't map it to a `T`.
pub fn from_str<'de, T: Deserialize<'de>>(input: &'de str) -> Result<T> {
    let mut deserializer = Deserializer::from_str(input);
    let t = T::deserialize(&mut deserializer)?;
    deserializer.skip_whitespace()?;
    match deserializer.peek() {
        Some((offset, _)) => Err(deserializer.err_at(offset, ErrorCode::TrailingCharacters)),
        None => Ok(t),
    }
}

/// A deserializer that knows how to parse JSON5 and map it on to types implementing
/// [`Deserialize`].
pub struct Deserializer<'de> {
    input: &'de str,
    char_indices: Peekable<CharIndices<'de>>,
}

impl<'de> Deserializer<'de> {
    /// Construct a deserializer that will read from the given JSON5 string.
    #[expect(
        clippy::should_implement_trait,
        reason = "Serde convention: https://serde.rs/conventions.html"
    )]
    #[must_use]
    pub fn from_str(input: &'de str) -> Self {
        Self {
            input,
            char_indices: input.char_indices().peekable(),
        }
    }
}

impl<'de> Deserializer<'de> {
    fn next(&mut self) -> Option<(usize, char)> {
        self.char_indices.next()
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.char_indices.peek().copied()
    }

    fn next_or(&mut self, eof: ErrorCode) -> Result<(usize, char)> {
        self.next().ok_or_else(|| Error::new(eof))
    }

    fn peek_or(&mut self, eof: ErrorCode) -> Result<(usize, char)> {
        self.peek().ok_or_else(|| Error::new(eof))
    }

    fn expect_char(
        &mut self,
        expected: char,
        eof: ErrorCode,
        unexpected: ErrorCode,
    ) -> Result<usize> {
        let (offset, c) = self.next_or(eof)?;
        if c != expected {
            return Err(self.err_at(offset, unexpected));
        }
        Ok(offset)
    }

    fn expect_str(
        &mut self,
        expected: &str,
        eof: ErrorCode,
        unexpected: ErrorCode,
    ) -> Result<usize> {
        let mut chars = expected.chars();
        let offset = self.expect_char(
            chars.next().expect("expecting at least one character"),
            eof,
            unexpected,
        )?;
        for e in chars {
            self.expect_char(e, eof, unexpected)?;
        }
        Ok(offset)
    }

    fn expect_collection_end(
        &mut self,
        close: char,
        eof: ErrorCode,
        unexpected: ErrorCode,
    ) -> Result<()> {
        self.skip_whitespace()?;
        let (offset, c) = self.next_or(eof)?;
        match c {
            c if c == close => Ok(()),
            ',' => {
                self.skip_whitespace()?;
                self.expect_char(close, eof, unexpected)?;
                Ok(())
            }
            _ => Err(self.err_at(offset, unexpected)),
        }
    }

    // https://spec.json5.org/#white-space
    fn skip_whitespace(&mut self) -> Result<()> {
        while let Some((_, c)) = self.peek() {
            match c {
                _ if crate::char::is_json5_whitespace(c) => {
                    self.next();
                }
                '/' => {
                    self.next();
                    self.skip_comment()?;
                }
                _ => {
                    break;
                }
            }
        }
        Ok(())
    }

    // https://spec.json5.org/#comments
    fn skip_comment(&mut self) -> Result<()> {
        let (offset, c) = self.next_or(ErrorCode::EofParsingComment)?;
        match c {
            '/' => {
                while let Some((_, c)) = self.next() {
                    if crate::char::is_json5_line_terminator(c) {
                        break;
                    }
                }
            }
            '*' => {
                while let Some((_, c)) = self.next() {
                    if c == '*' && self.peek().is_some_and(|(_, c)| c == '/') {
                        self.next();
                        break;
                    }
                }
            }
            _ => {
                return Err(self.err_at(offset, ErrorCode::ExpectedComment));
            }
        }
        Ok(())
    }

    fn parse_null(&mut self) -> Result<usize> {
        self.skip_whitespace()?;
        let (offset, _) = self.peek_or(ErrorCode::EofParsingNull)?;
        self.expect_str("null", ErrorCode::EofParsingNull, ErrorCode::ExpectedNull)?;
        Ok(offset)
    }

    fn parse_bool(&mut self) -> Result<(usize, bool)> {
        self.skip_whitespace()?;

        match self.next_or(ErrorCode::EofParsingBool)? {
            (offset, 't') => {
                self.expect_str("rue", ErrorCode::EofParsingBool, ErrorCode::ExpectedBool)?;
                Ok((offset, true))
            }
            (offset, 'f') => {
                self.expect_str("alse", ErrorCode::EofParsingBool, ErrorCode::ExpectedBool)?;
                Ok((offset, false))
            }
            (offset, _) => Err(self.err_at(offset, ErrorCode::ExpectedBool)),
        }
    }

    // https://spec.json5.org/#numbers
    fn parse_number(&mut self) -> Result<(usize, NumberResult)> {
        self.skip_whitespace()?;

        let (start, _) = self.peek_or(ErrorCode::EofParsingNumber)?;

        let neg = match self.peek_or(ErrorCode::EofParsingNumber)? {
            (_, '+') => {
                self.next();
                false
            }
            (_, '-') => {
                self.next();
                true
            }
            _ => false,
        };

        match self.next_or(ErrorCode::EofParsingNumber)? {
            (_, 'I') => {
                self.expect_str(
                    "nfinity",
                    ErrorCode::EofParsingNumber,
                    ErrorCode::ExpectedNumber,
                )?;
                if neg {
                    Ok((start, NumberResult::F64(-f64::INFINITY)))
                } else {
                    Ok((start, NumberResult::F64(f64::INFINITY)))
                }
            }
            (_, 'N') => {
                self.expect_str("aN", ErrorCode::EofParsingNumber, ErrorCode::ExpectedNumber)?;
                if neg {
                    Ok((start, NumberResult::F64(-f64::NAN)))
                } else {
                    Ok((start, NumberResult::F64(f64::NAN)))
                }
            }
            (_, '0') => match self.peek() {
                Some((_, 'x' | 'X')) => {
                    self.next();
                    self.parse_hex_number(neg, start).map(|n| (start, n))
                }
                Some((offset, '.' | 'e' | 'E')) => self
                    .parse_decimal_number(neg, start, offset)
                    .map(|n| (start, n)),
                Some((_, '0'..='9')) => Err(self.err_at(start, ErrorCode::LeadingZero)),
                _ => Ok((start, NumberResult::U128(0))),
            },
            (offset, '.' | '1'..='9') => self
                .parse_decimal_number(neg, start, offset)
                .map(|n| (start, n)),
            (offset, _) => Err(self.err_at(offset, ErrorCode::ExpectedNumber)),
        }
    }

    // Aside from the representation of Infinity, NaN, and hex numbers, which are handled in
    // parse_number, the f64, i64, and u64 implementations of FromStr implement exactly the format
    // we need.
    fn parse_decimal_number(
        &mut self,
        neg: bool,
        start: usize,
        mut offset: usize,
    ) -> Result<NumberResult> {
        while let Some((o, c)) = self.peek()
            && matches!(c, '+' | '-' | '.' | 'e' | 'E' | '0'..='9')
        {
            self.next();
            offset = o;
        }
        if self.input[start..=offset].contains(['.', 'e', 'E']) {
            // https://doc.rust-lang.org/std/primitive.f64.html#method.from_str
            Ok(NumberResult::F64(self.parse_from_str(start, offset)?))
        } else if neg {
            // https://doc.rust-lang.org/std/primitive.i64.html#method.from_str
            Ok(NumberResult::I128(self.parse_from_str(start, offset)?))
        } else {
            // https://doc.rust-lang.org/std/primitive.u64.html#method.from_str
            Ok(NumberResult::U128(self.parse_from_str(start, offset)?))
        }
    }

    fn parse_from_str<N>(&self, start: usize, offset: usize) -> Result<N>
    where
        N: FromStr,
        N::Err: Display,
    {
        self.input[start..=offset]
            .parse()
            .map_err(|err: N::Err| self.custom_err_at(start, err))
    }

    fn parse_hex_number(&mut self, neg: bool, start: usize) -> Result<NumberResult> {
        let (offset, c) = self.next_or(ErrorCode::EofParsingNumber)?;
        if !c.is_ascii_hexdigit() {
            return Err(self.err_at(offset, ErrorCode::ExpectedNumber));
        }
        let mut n = u128::from(c.to_digit(16).expect("c is ascii hexdigit"));

        while let Some((offset, c)) = self.peek()
            && c.is_ascii_hexdigit()
        {
            self.next();
            n = n
                .checked_mul(16)
                .and_then(|n| {
                    n.checked_add(u128::from(c.to_digit(16).expect("c is ascii hexdigit")))
                })
                .ok_or(self.err_at(offset, ErrorCode::OverflowParsingNumber))?;
        }

        if neg {
            // Special case for i128::MIN because -i128::MIN = i128::MAX + 1 doesn't fit in an i128.
            if n == 0x8000_0000_0000_0000_0000_0000_0000_0000 {
                Ok(NumberResult::I128(i128::MIN))
            } else {
                Ok(NumberResult::I128(
                    -i128::try_from(n).map_err(|err| self.custom_err_at(start, err))?,
                ))
            }
        } else {
            Ok(NumberResult::U128(n))
        }
    }

    // https://spec.json5.org/#strings
    fn parse_string(&mut self) -> Result<(usize, StringResult<'de>)> {
        self.skip_whitespace()?;

        let (offset, c) = self.next_or(ErrorCode::EofParsingString)?;
        if c == '"' || c == '\'' {
            self.parse_string_characters(c).map(|s| (offset, s))
        } else {
            Err(self.err_at(offset, ErrorCode::ExpectedString))
        }
    }

    fn parse_string_characters(&mut self, delimiter: char) -> Result<StringResult<'de>> {
        let mut owned = None;
        let (start, _) = self.peek_or(ErrorCode::EofParsingString)?;

        loop {
            let (offset, c) = self.next_or(ErrorCode::EofParsingString)?;

            if c == delimiter {
                if let Some(owned) = owned {
                    return Ok(StringResult::Owned(owned));
                }
                return Ok(StringResult::Borrowed(&self.input[start..offset]));
            } else if c == '\u{000A}' || c == '\u{000D}' {
                // LineTerminator is forbidden except U+2028 and U+2029 are explicitly allowed.
                return Err(self.err_at(offset, ErrorCode::LineTerminatorInString));
            } else if c == '\\' {
                let owned = owned.get_or_insert(self.input[start..offset].to_owned());
                if let Some(c) = self.parse_escape_sequence(offset)? {
                    owned.push(c);
                }
            } else if let Some(owned) = &mut owned {
                owned.push(c);
            }
        }
    }

    // https://262.ecma-international.org/5.1/#sec-7.8.4
    fn parse_escape_sequence(&mut self, offset: usize) -> Result<Option<char>> {
        let (_, c) = self.next_or(ErrorCode::EofParsingEscapeSequence)?;
        match c {
            // LineTerminatorSequence
            _ if crate::char::is_json5_line_terminator(c) => {
                if c == '\u{000D}' && self.peek().is_some_and(|(_, c)| c == '\u{000A}') {
                    self.next();
                }
                Ok(None)
            }

            // SingleEscapeCharacter (Table 4 — String Single Character Escape Sequences)
            'b' => Ok(Some('\u{0008}')),
            't' => Ok(Some('\u{0009}')),
            'n' => Ok(Some('\u{000A}')),
            'v' => Ok(Some('\u{000B}')),
            'f' => Ok(Some('\u{000C}')),
            'r' => Ok(Some('\u{000D}')),

            '0' => {
                if self.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
                    return Err(self.err_at(offset, ErrorCode::InvalidEscapeSequence));
                }
                Ok(Some('\u{0000}'))
            }

            '1'..'9' => Err(self.err_at(offset, ErrorCode::InvalidEscapeSequence)),

            'x' => Ok(Some(self.parse_hex_escape_sequence(offset)?)),

            'u' => Ok(Some(self.parse_unicode_escape_sequence(offset)?)),

            c => Ok(Some(c)),
        }
    }

    fn parse_hex_escape_sequence(&mut self, offset: usize) -> Result<char> {
        char::try_from(self.parse_escape_sequence_digits(offset, 2)?)
            .map_err(|err| self.custom_err_at(offset, err))
    }

    fn parse_unicode_escape_sequence(&mut self, offset: usize) -> Result<char> {
        let a = self.parse_escape_sequence_digits(offset, 4)?;
        if let Ok(c) = char::try_from(a) {
            return Ok(c);
        }

        // "To escape an extended character that is not in the Basic Multilingual Plane, the
        // character is represented as a 12-character sequence, encoding the UTF-16 surrogate pair."
        // – https://spec.json5.org/#escapes
        self.expect_str(
            "\\u",
            ErrorCode::EofParsingEscapeSequence,
            ErrorCode::InvalidEscapeSequence,
        )?;
        let b = self.parse_escape_sequence_digits(offset, 4)?;

        let mut chars = char::decode_utf16([
            u16::try_from(a).map_err(|err| self.custom_err_at(offset, err))?,
            u16::try_from(b).map_err(|err| self.custom_err_at(offset, err))?,
        ]);
        let c = chars
            .next()
            .ok_or_else(|| self.err_at(offset, ErrorCode::InvalidEscapeSequence))?
            .map_err(|err| self.custom_err_at(offset, err))?;

        if chars.next().is_none() {
            Ok(c)
        } else {
            Err(self.err_at(offset, ErrorCode::InvalidEscapeSequence))
        }
    }

    fn parse_escape_sequence_digits(&mut self, offset: usize, length: usize) -> Result<u32> {
        let mut value = 0;
        for _ in 0..length {
            let (_, c) = self.next_or(ErrorCode::EofParsingEscapeSequence)?;
            if !c.is_ascii_hexdigit() {
                return Err(self.err_at(offset, ErrorCode::InvalidEscapeSequence));
            }
            value = value * 16 + c.to_digit(16).expect("c.is_ascii_hexdigit");
        }
        Ok(value)
    }

    // https://spec.json5.org/#objects
    fn parse_key(&mut self) -> Result<(usize, StringResult<'de>)> {
        self.skip_whitespace()?;

        match self.peek_or(ErrorCode::EofParsingObject)? {
            (_, '"' | '\'') => self.parse_string(),
            (offset, _) => self.parse_identifier().map(|i| (offset, i)),
        }
    }

    // https://262.ecma-international.org/5.1/#sec-7.6
    fn parse_identifier(&mut self) -> Result<StringResult<'de>> {
        let mut owned = None;
        let (start, _) = self.peek_or(ErrorCode::EofParsingIdentifier)?;

        loop {
            let (offset, c) = self.peek_or(ErrorCode::EofParsingIdentifier)?;

            if c == '\\' {
                self.next();
                let owned = owned.get_or_insert(self.input[start..offset].to_owned());
                self.expect_char(
                    'u',
                    ErrorCode::EofParsingIdentifier,
                    ErrorCode::InvalidEscapeSequence,
                )?;
                let c = self.parse_unicode_escape_sequence(offset)?;
                if offset == start {
                    if !crate::char::is_json5_identifier_start(c) {
                        return Err(self.err_at(offset, ErrorCode::ExpectedIdentifier));
                    }
                } else if !crate::char::is_json5_identifier(c) {
                    return Err(self.err_at(offset, ErrorCode::ExpectedIdentifier));
                }
                owned.push(c);
                continue;
            }

            if offset == start {
                if !crate::char::is_json5_identifier_start(c) {
                    return Err(self.err_at(offset, ErrorCode::ExpectedIdentifier));
                }
            } else if !crate::char::is_json5_identifier(c) {
                return Ok(match owned {
                    Some(owned) => StringResult::Owned(owned),
                    None => StringResult::Borrowed(&self.input[start..offset]),
                });
            }

            self.next();
            if let Some(owned) = &mut owned {
                owned.push(c);
            }
        }
    }

    fn decode_hex(&self, offset: usize, s: &str) -> Result<Vec<u8>> {
        let mut chars = s.chars();
        let mut bytes = Vec::new();
        while let Some(a) = chars.next() {
            match a
                .to_digit(16)
                .and_then(|a| chars.next().and_then(|b| b.to_digit(16)).map(|b| (a, b)))
            {
                Some((a, b)) => {
                    bytes.push(u8::try_from(a * 16 + b).expect("two hex digits fit in a u8"));
                }
                None => return Err(self.err_at(offset, ErrorCode::InvalidBytes)),
            }
        }
        Ok(bytes)
    }

    fn err_at(&self, offset: usize, code: ErrorCode) -> Error {
        Error::new_at(Position::from_offset(offset, self.input), code)
    }

    fn custom_err_at<T: Display>(&self, offset: usize, msg: T) -> Error {
        Error::custom_at(Position::from_offset(offset, self.input), msg)
    }

    fn with_position(&self, err: Error, offset: usize) -> Error {
        err.with_position(Position::from_offset(offset, self.input))
    }
}

macro_rules! deserialize_number {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let (offset, number) = self.parse_number()?;
            match number {
                NumberResult::U128(u) => {
                    if let Ok(u) = u64::try_from(u) {
                        visitor.visit_u64(u)
                    } else {
                        visitor.visit_u128(u)
                    }
                }
                NumberResult::I128(i) => {
                    if let Ok(i) = i64::try_from(i) {
                        visitor.visit_i64(i)
                    } else {
                        visitor.visit_i128(i)
                    }
                }
                NumberResult::F64(f) => visitor.visit_f64(f),
            }
            .map_err(|err| self.with_position(err, offset))
        }
    };
}

macro_rules! deserialize_string {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let (offset, s) = self.parse_string()?;
            match s {
                StringResult::Borrowed(borrowed) => visitor.visit_borrowed_str(borrowed),
                StringResult::Owned(owned) => visitor.visit_string(owned),
            }
            .map_err(|err| self.with_position(err, offset))
        }
    };
}

macro_rules! deserialize_bytes {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let (offset, s) = self.parse_string()?;
            visitor
                .visit_byte_buf(self.decode_hex(offset, &s)?)
                .map_err(|err| self.with_position(err, offset))
        }
    };
}

macro_rules! deserialize_collection {
    (
        $method:ident,
        $visit:ident,
        $access:ident,
        $open:expr,
        $close:expr,
        $eof:expr,
        $expected_opening:expr,
        $expected_closing:expr,
    ) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            self.skip_whitespace()?;
            let offset = self.expect_char($open, $eof, $expected_opening)?;
            let value = visitor
                .$visit($access {
                    de: self,
                    first: true,
                })
                .map_err(|err| self.with_position(err, offset))?;
            self.expect_collection_end($close, $eof, $expected_closing)?;
            Ok(value)
        }
    };
}

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    deserialize_number!(deserialize_u128);
    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_i128);
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    deserialize_string!(deserialize_str);
    deserialize_string!(deserialize_string);
    deserialize_string!(deserialize_char);
    deserialize_string!(deserialize_identifier);

    deserialize_bytes!(deserialize_bytes);
    deserialize_bytes!(deserialize_byte_buf);

    deserialize_collection!(
        deserialize_seq,
        visit_seq,
        SeqAccess,
        '[',
        ']',
        ErrorCode::EofParsingArray,
        ErrorCode::ExpectedOpeningBracket,
        ErrorCode::ExpectedClosingBracket,
    );
    deserialize_collection!(
        deserialize_map,
        visit_map,
        MapAccess,
        '{',
        '}',
        ErrorCode::EofParsingObject,
        ErrorCode::ExpectedOpeningBrace,
        ErrorCode::ExpectedClosingBrace,
    );

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (offset, b) = self.parse_bool()?;
        visitor
            .visit_bool(b)
            .map_err(|err| self.with_position(err, offset))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace()?;
        let (offset, c) = self.peek_or(ErrorCode::EofParsingValue)?;
        if c == 'n' {
            self.parse_null()?;
            visitor.visit_none()
        } else {
            visitor.visit_some(&mut *self)
        }
        .map_err(|err| self.with_position(err, offset))
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let offset = self.parse_null()?;
        visitor
            .visit_unit()
            .map_err(|err| self.with_position(err, offset))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.skip_whitespace()?;
        match self.peek_or(ErrorCode::EofParsingValue)? {
            (offset, '{') => {
                self.next();
                let value = visitor
                    .visit_enum(VariantAccess { de: self })
                    .map_err(|err| self.with_position(err, offset))?;
                self.expect_collection_end(
                    '}',
                    ErrorCode::EofParsingObject,
                    ErrorCode::ExpectedClosingBrace,
                )?;
                Ok(value)
            }
            (offset, '"' | '\'') => visitor
                .visit_enum(UnitVariantAccess {
                    de: self,
                    map_key: false,
                })
                .map_err(|err| self.with_position(err, offset)),
            (c, _) => Err(self.err_at(c, ErrorCode::ExpectedStringOrObject)),
        }
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip_whitespace()?;
        match self.peek_or(ErrorCode::EofParsingValue)? {
            (_, 'n') => self.deserialize_unit(visitor),
            (_, 't' | 'f') => self.deserialize_bool(visitor),
            (_, '"' | '\'') => self.deserialize_str(visitor),
            (_, '+' | '-' | '.' | 'I' | 'N' | '0'..='9') => self.deserialize_f64(visitor),
            (_, '[') => self.deserialize_seq(visitor),
            (_, '{') => self.deserialize_map(visitor),
            (offset, _) => Err(self.err_at(offset, ErrorCode::ExpectedValue)),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_any(visitor)
    }
}

struct SeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'de> serde::de::SeqAccess<'de> for SeqAccess<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.de.skip_whitespace()?;
        if self.de.peek().is_some_and(|(_, c)| c == ']') {
            return Ok(None);
        }

        if !self.first {
            self.de
                .expect_char(',', ErrorCode::EofParsingArray, ErrorCode::ExpectedComma)?;

            self.de.skip_whitespace()?;
            if self.de.peek().is_some_and(|(_, c)| c == ']') {
                return Ok(None);
            }
        }
        self.first = false;

        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct MapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'de> serde::de::MapAccess<'de> for MapAccess<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.de.skip_whitespace()?;
        if self.de.peek().is_some_and(|(_, c)| c == '}') {
            return Ok(None);
        }

        if !self.first {
            self.de
                .expect_char(',', ErrorCode::EofParsingObject, ErrorCode::ExpectedComma)?;

            self.de.skip_whitespace()?;
            if self.de.peek().is_some_and(|(_, c)| c == '}') {
                return Ok(None);
            }
        }
        self.first = false;

        seed.deserialize(MapKey { de: self.de }).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        self.de.skip_whitespace()?;
        self.de
            .expect_char(':', ErrorCode::EofParsingObject, ErrorCode::ExpectedColon)?;
        seed.deserialize(&mut *self.de)
    }
}

struct MapKey<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

macro_rules! deserialize_key_from_str {
    ($method:ident, $visit:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let (offset, key) = self.de.parse_key()?;
            visitor
                .$visit(from_str(&key)?)
                .map_err(|err| self.de.with_position(err, offset))
        }
    };
}

macro_rules! deserialize_string_key {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let (offset, key) = self.de.parse_key()?;
            match key {
                StringResult::Borrowed(borrowed) => visitor.visit_borrowed_str(borrowed),
                StringResult::Owned(owned) => visitor.visit_string(owned),
            }
            .map_err(|err| self.de.with_position(err, offset))
        }
    };
}

macro_rules! deserialize_bytes_key {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let (offset, s) = self.de.parse_key()?;
            visitor
                .visit_byte_buf(self.de.decode_hex(offset, &s)?)
                .map_err(|err| self.de.with_position(err, offset))
        }
    };
}

impl<'de> serde::de::Deserializer<'de> for MapKey<'_, 'de> {
    type Error = Error;

    deserialize_key_from_str!(deserialize_bool, visit_bool);
    deserialize_key_from_str!(deserialize_u8, visit_u8);
    deserialize_key_from_str!(deserialize_u16, visit_u16);
    deserialize_key_from_str!(deserialize_u32, visit_u32);
    deserialize_key_from_str!(deserialize_u64, visit_u64);
    deserialize_key_from_str!(deserialize_u128, visit_u128);
    deserialize_key_from_str!(deserialize_i8, visit_i8);
    deserialize_key_from_str!(deserialize_i16, visit_i16);
    deserialize_key_from_str!(deserialize_i32, visit_i32);
    deserialize_key_from_str!(deserialize_i64, visit_i64);
    deserialize_key_from_str!(deserialize_i128, visit_i128);
    deserialize_key_from_str!(deserialize_f32, visit_f32);
    deserialize_key_from_str!(deserialize_f64, visit_f64);

    deserialize_string_key!(deserialize_any);
    deserialize_string_key!(deserialize_ignored_any);
    deserialize_string_key!(deserialize_str);
    deserialize_string_key!(deserialize_string);
    deserialize_string_key!(deserialize_char);
    deserialize_string_key!(deserialize_identifier);

    deserialize_bytes_key!(deserialize_bytes);
    deserialize_bytes_key!(deserialize_byte_buf);

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (offset, key) = self.de.parse_key()?;
        from_str::<()>(&key)?;
        visitor
            .visit_unit()
            .map_err(|err| self.de.with_position(err, offset))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // Consider keys to always be Some, otherwise we don't know if "null" is None or
        // Some("null").
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let (offset, _) = self.de.peek_or(ErrorCode::EofParsingObject)?;
        visitor
            .visit_enum(UnitVariantAccess {
                de: self.de,
                map_key: true,
            })
            .map_err(|err| self.de.with_position(err, offset))
    }

    forward_to_deserialize_any! { seq tuple tuple_struct map struct }
}

struct VariantAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de> serde::de::EnumAccess<'de> for VariantAccess<'_, 'de> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(MapKey { de: &mut *self.de })?;
        self.de.skip_whitespace()?;
        self.de
            .expect_char(':', ErrorCode::EofParsingObject, ErrorCode::ExpectedColon)?;
        Ok((variant, self))
    }
}

impl<'de> serde::de::VariantAccess<'de> for VariantAccess<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        serde::de::Deserialize::deserialize(self.de)
    }

    fn newtype_variant_seed<T: serde::de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        serde::de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        serde::de::Deserializer::deserialize_struct(self.de, "", fields, visitor)
    }
}

struct UnitVariantAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    map_key: bool,
}

impl<'de> serde::de::EnumAccess<'de> for UnitVariantAccess<'_, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = if self.map_key {
            seed.deserialize(MapKey { de: &mut *self.de })?
        } else {
            seed.deserialize(&mut *self.de)?
        };
        Ok((variant, self))
    }
}

impl<'de> serde::de::VariantAccess<'de> for UnitVariantAccess<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _: usize, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(self, _: &'static [&'static str], _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

enum StringResult<'de> {
    Borrowed(&'de str),
    Owned(String),
}

impl Deref for StringResult<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            StringResult::Borrowed(borrowed) => borrowed,
            StringResult::Owned(owned) => owned,
        }
    }
}

enum NumberResult {
    U128(u128),
    I128(i128),
    F64(f64),
}
