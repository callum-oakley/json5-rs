use std::{
    fmt::Display,
    iter::Peekable,
    str::{CharIndices, FromStr},
};

use serde::{Deserialize, de::Visitor};

use crate::{Error, ErrorCode, Position, Result};

pub fn from_str<'de, T: Deserialize<'de>>(input: &'de str) -> Result<T> {
    let mut deserializer = Deserializer::from_str(input);
    let t = T::deserialize(&mut deserializer)?;
    match deserializer.peek() {
        Some((offset, _)) => Err(deserializer.err_at(offset, ErrorCode::TrailingCharacters)),
        None => Ok(t),
    }
}

pub struct Deserializer<'de> {
    input: &'de str,
    char_indices: Peekable<CharIndices<'de>>,
}

impl<'de> Deserializer<'de> {
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

    fn next_or(&mut self, code: ErrorCode) -> Result<(usize, char)> {
        self.next().ok_or(Error::new(code))
    }

    fn peek_or(&mut self, code: ErrorCode) -> Result<(usize, char)> {
        self.peek().ok_or(Error::new(code))
    }

    fn expect_str(&mut self, ident: &str, eof: &ErrorCode, unexpected: ErrorCode) -> Result<()> {
        for expected in ident.chars() {
            let (offset, c) = self.next_or(eof.clone())?;
            if c != expected {
                return Err(self.err_at(offset, unexpected));
            }
        }
        Ok(())
    }

    fn parse_null(&mut self) -> Result<()> {
        self.expect_str("null", &ErrorCode::EofParsingNull, ErrorCode::ExpectedNull)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        match self.next_or(ErrorCode::EofParsingBool)? {
            (_, 't') => {
                self.expect_str("rue", &ErrorCode::EofParsingBool, ErrorCode::ExpectedBool)?;
                Ok(true)
            }
            (_, 'f') => {
                self.expect_str("alse", &ErrorCode::EofParsingBool, ErrorCode::ExpectedBool)?;
                Ok(false)
            }
            (offset, _) => Err(self.err_at(offset, ErrorCode::ExpectedBool)),
        }
    }

    // https://spec.json5.org/#numbers
    fn parse_number(&mut self) -> Result<NumberResult> {
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
                    &ErrorCode::EofParsingNumber,
                    ErrorCode::ExpectedNumber,
                )?;
                if neg {
                    Ok(NumberResult::F64(-f64::INFINITY))
                } else {
                    Ok(NumberResult::F64(f64::INFINITY))
                }
            }
            (_, 'N') => {
                self.expect_str(
                    "aN",
                    &ErrorCode::EofParsingNumber,
                    ErrorCode::ExpectedNumber,
                )?;
                if neg {
                    Ok(NumberResult::F64(-f64::NAN))
                } else {
                    Ok(NumberResult::F64(f64::NAN))
                }
            }
            (_, '0') => match self.peek() {
                Some((_, 'x' | 'X')) => {
                    self.next();
                    self.parse_hex_number(neg, start)
                }
                Some((offset, '.' | 'e' | 'E')) => self.parse_decimal_number(neg, start, offset),
                Some((_, '0'..='9')) => Err(self.err_at(start, ErrorCode::LeadingZero)),
                _ => Ok(NumberResult::U64(0)),
            },
            (offset, '.' | '1'..='9') => self.parse_decimal_number(neg, start, offset),
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
            Ok(NumberResult::I64(self.parse_from_str(start, offset)?))
        } else {
            // https://doc.rust-lang.org/std/primitive.u64.html#method.from_str
            Ok(NumberResult::U64(self.parse_from_str(start, offset)?))
        }
    }

    fn parse_from_str<N>(&self, start: usize, offset: usize) -> Result<N>
    where
        N: FromStr,
        N::Err: Display,
    {
        self.input[start..=offset]
            .parse()
            .map_err(|err: N::Err| self.err_at(start, ErrorCode::Message(err.to_string())))
    }

    fn parse_hex_number(&mut self, neg: bool, start: usize) -> Result<NumberResult> {
        let (offset, c) = self.next_or(ErrorCode::EofParsingNumber)?;
        if !c.is_ascii_hexdigit() {
            return Err(self.err_at(offset, ErrorCode::ExpectedNumber));
        }
        let mut n = u64::from(c.to_digit(16).expect("c is ascii hexdigit"));

        while let Some((offset, c)) = self.peek()
            && c.is_ascii_hexdigit()
        {
            self.next();
            n = n
                .checked_mul(16)
                .and_then(|n| {
                    n.checked_add(u64::from(c.to_digit(16).expect("c is ascii hexdigit")))
                })
                .ok_or(self.err_at(offset, ErrorCode::OverflowParsingNumber))?;
        }

        if neg {
            // Special case for i64::MIN because -i64::MIN = i64::MAX + 1, so we'll overflow if we
            // try to cast -i64::MIN from a u64 to an i64.
            if n == 0x8000_0000_0000_0000 {
                Ok(NumberResult::I64(i64::MIN))
            } else {
                Ok(NumberResult::I64(-i64::try_from(n).map_err(|err| {
                    self.err_at(start, ErrorCode::Message(err.to_string()))
                })?))
            }
        } else {
            Ok(NumberResult::U64(n))
        }
    }

    // https://spec.json5.org/#strings
    fn parse_string(&mut self) -> Result<StringResult<'de>> {
        let (offset, c) = self.next_or(ErrorCode::EofParsingString)?;
        if c == '"' || c == '\'' {
            self.parse_string_characters(c)
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
                if let Some(c) = self.parse_escape_sequence()? {
                    owned.push(c);
                }
            } else if let Some(owned) = &mut owned {
                owned.push(c);
            }
        }
    }

    // https://262.ecma-international.org/5.1/#sec-7.8.4
    fn parse_escape_sequence(&mut self) -> Result<Option<char>> {
        let (offset, c) = self.next_or(ErrorCode::EofParsingString)?;
        match c {
            // LineTerminatorSequence
            '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => {
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
                if let Some((offset, c)) = self.peek()
                    && c.is_ascii_digit()
                {
                    return Err(self.err_at(offset, ErrorCode::InvalidEscapeSequence));
                }
                Ok(Some('\u{0000}'))
            }

            '1'..'9' => Err(self.err_at(offset, ErrorCode::InvalidEscapeSequence)),

            'x' => Ok(Some(self.parse_hex_escape_sequence(2)?)),

            'u' => Ok(Some(self.parse_hex_escape_sequence(4)?)),

            c => Ok(Some(c)),
        }
    }

    fn parse_hex_escape_sequence(&mut self, length: usize) -> Result<char> {
        let mut value = 0;
        for _ in 0..length {
            let (offset, c) = self.next_or(ErrorCode::EofParsingString)?;
            if !c.is_ascii_hexdigit() {
                return Err(self.err_at(offset, ErrorCode::InvalidEscapeSequence));
            }
            value = value * 16 + c.to_digit(16).expect("c.is_ascii_hexdigit");
        }
        Ok(char::from_u32(value).expect("escape sequence isn't long enough to overflow a char"))
    }

    fn err_at(&self, offset: usize, code: ErrorCode) -> Error {
        Error::new_at(Position::from_offset(offset, self.input), code)
    }
}

macro_rules! deserialize_number {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            match self.parse_number()? {
                NumberResult::U64(u) => visitor.visit_u64(u),
                NumberResult::I64(i) => visitor.visit_i64(i),
                NumberResult::F64(f) => visitor.visit_f64(f),
            }
        }
    };
}

macro_rules! deserialize_string {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            match self.parse_string()? {
                StringResult::Borrowed(borrowed) => visitor.visit_borrowed_str(borrowed),
                StringResult::Owned(owned) => visitor.visit_string(owned),
            }
        }
    };
}

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    deserialize_string!(deserialize_str);
    deserialize_string!(deserialize_string);
    deserialize_string!(deserialize_char);

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.peek().is_some_and(|(_, c)| c == 'n') {
            self.parse_null()?;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.parse_null()?;
        visitor.visit_unit()
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

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        todo!()
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.peek_or(ErrorCode::EofParsingValue)? {
            (_, 'n') => self.deserialize_unit(visitor),
            (_, 't' | 'f') => self.deserialize_bool(visitor),
            (_, '"' | '\'') => self.deserialize_str(visitor),
            (_, '+' | '-' | '.' | 'I' | 'N' | '0'..='9') => self.deserialize_f64(visitor),
            _ => todo!(),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_any(visitor)
    }
}

enum StringResult<'de> {
    Borrowed(&'de str),
    Owned(String),
}

enum NumberResult {
    U64(u64),
    I64(i64),
    F64(f64),
}
