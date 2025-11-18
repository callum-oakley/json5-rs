use std::{iter::Peekable, str::CharIndices};

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

    // https://spec.json5.org/#strings
    fn parse_string(&mut self) -> Result<StringResult<'de>> {
        let (offset, c) = self.next_or(ErrorCode::EofParsingString)?;
        if !matches!(c, '"' | '\'') {
            return Err(self.err_at(offset, ErrorCode::ExpectedString));
        }
        self.parse_string_characters(c)
    }

    // https://spec.json5.org/#numbers
    fn parse_number(&mut self) -> Result<f64> {
        todo!()
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
            } else if matches!(c, '\u{000A}' | '\u{000D}') {
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

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_or(ErrorCode::EofParsingValue)? {
            (_, 'n') => self.deserialize_unit(visitor),
            (_, 't' | 'f') => self.deserialize_bool(visitor),
            (_, '"' | '\'') => self.deserialize_str(visitor),
            _ => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.parse_string()? {
            StringResult::Borrowed(borrowed) => visitor.visit_borrowed_str(borrowed),
            StringResult::Owned(owned) => visitor.visit_string(owned),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek().is_some_and(|(_, c)| c == 'n') {
            self.parse_null()?;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_null()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}

enum StringResult<'de> {
    Borrowed(&'de str),
    Owned(String),
}
