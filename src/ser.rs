use std::io::Write;

use serde::{Serialize, ser::Impossible};

use crate::{Error, ErrorCode, error::Result};

/// Serialize a type implementing [`Serialize`] to a JSON5 string.
///
/// # Example
/// ```
/// use serde_derive::Serialize;
///
/// #[derive(Serialize)]
/// struct Config<'a> {
///     foo: u32,
///     bar: &'a str,
/// }
///
/// let config = Config {
///     foo: 42,
///     bar: "baz",
/// };
///
/// assert_eq!(&json5::to_string(&config)?, "{
///   foo: 42,
///   bar: \"baz\",
/// }");
/// # Ok::<(), json5::Error>(())
/// ```
///
/// # Errors
/// Fails if we can't express `T` in JSON5 (e.g. we try to serialize an object key without an
/// obvious string representation).
pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    let mut w = Vec::new();
    to_writer(&mut w, value)?;
    #[expect(clippy::missing_panics_doc)]
    Ok(String::from_utf8(w).expect("we only write valid UTF-8"))
}

/// Serialize a type implementing [`Serialize`] to JSON5 and write it to the given writer.
///
/// # Errors
/// Fails if we can't express `T` in JSON5 (e.g. we try to serialize an object key without an
/// obvious string representation) or if there's an error writing to the writer.
pub fn to_writer<T: Serialize, W: Write>(w: W, value: &T) -> Result<()> {
    value.serialize(&mut Serializer::new(w))
}

/// A serializer that knows how to serialize types implementing [`Serialize`] as JSON5.
pub struct Serializer<W: Write> {
    w: W,
    depth: usize,
}

impl<W: Write> Serializer<W> {
    pub fn new(w: W) -> Self {
        Self { w, depth: 0 }
    }
}

macro_rules! serialize_display {
    ($method:ident, $type:ty) => {
        fn $method(self, v: $type) -> Result<Self::Ok> {
            write!(self.w, "{v}").map_err(Into::into)
        }
    };
}

macro_rules! serialize_float {
    ($method:ident, $type:ty) => {
        fn $method(self, v: $type) -> Result<Self::Ok> {
            match (v.is_nan(), v.is_infinite(), v.is_sign_negative()) {
                (true, false, false) => write!(self.w, "NaN"),
                (true, false, true) => write!(self.w, "-NaN"),
                (false, true, false) => write!(self.w, "Infinity"),
                (false, true, true) => write!(self.w, "-Infinity"),
                _ => write!(self.w, "{v}"),
            }
            .map_err(Into::into)
        }
    };
}

impl<'a, W: Write> serde::ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeCollection<'a, W>;
    type SerializeTuple = SerializeCollection<'a, W>;
    type SerializeTupleStruct = SerializeCollection<'a, W>;
    type SerializeTupleVariant = SerializeCollection<'a, W>;
    type SerializeMap = SerializeCollection<'a, W>;
    type SerializeStruct = SerializeCollection<'a, W>;
    type SerializeStructVariant = SerializeCollection<'a, W>;

    serialize_display!(serialize_bool, bool);
    serialize_display!(serialize_u8, u8);
    serialize_display!(serialize_u16, u16);
    serialize_display!(serialize_u32, u32);
    serialize_display!(serialize_u64, u64);
    serialize_display!(serialize_i8, i8);
    serialize_display!(serialize_i16, i16);
    serialize_display!(serialize_i32, i32);
    serialize_display!(serialize_i64, i64);
    serialize_float!(serialize_f32, f32);
    serialize_float!(serialize_f64, f64);

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        if v == '"' {
            write!(self.w, r#"'"'"#)
        } else if let Some(escaped) = crate::char::escape('"', v) {
            write!(self.w, r#""{escaped}""#)
        } else {
            write!(self.w, r#""{v}""#)
        }
        .map_err(Into::into)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let delimiter = if v.contains('"') && !v.contains('\'') {
            '\''
        } else {
            '"'
        };
        write!(self.w, "{delimiter}")?;
        for c in v.chars() {
            match crate::char::escape(delimiter, c) {
                Some(escaped) => write!(self.w, "{escaped}")?,
                None => write!(self.w, "{c}")?,
            }
        }
        write!(self.w, "{delimiter}")?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        write!(self.w, "\"")?;
        for b in v {
            write!(self.w, "{b:02x}")?;
        }
        write!(self.w, "\"")?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        write!(self.w, "null").map_err(Into::into)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, v: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        v.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        write!(self.w, "{{")?;
        self.depth += 1;
        write!(self.w, "\n{:indent$}", "", indent = self.depth * 2)?;
        MapKey::new(self).serialize_str(variant)?;
        write!(self.w, ": ")?;
        v.serialize(&mut *self)?;
        self.depth -= 1;
        write!(self.w, ",\n{:indent$}}}", "", indent = self.depth * 2)?;
        Ok(())
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        write!(self.w, "[")?;
        self.depth += 1;
        Ok(SerializeCollection::new(self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        write!(self.w, "{{")?;
        self.depth += 1;
        write!(self.w, "\n{:indent$}", "", indent = self.depth * 2)?;
        MapKey::new(self).serialize_str(variant)?;
        write!(self.w, ": ")?;
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        write!(self.w, "{{")?;
        self.depth += 1;
        Ok(SerializeCollection::new(self))
    }

    fn serialize_struct(self, _: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        write!(self.w, "{{")?;
        self.depth += 1;
        write!(self.w, "\n{:indent$}", "", indent = self.depth * 2)?;
        MapKey::new(self).serialize_str(variant)?;
        write!(self.w, ": ")?;
        self.serialize_map(Some(len))
    }
}

pub struct SerializeCollection<'a, W: Write> {
    ser: &'a mut Serializer<W>,
    empty: bool,
}

impl<'a, W: Write> SerializeCollection<'a, W> {
    fn new(ser: &'a mut Serializer<W>) -> Self {
        Self { ser, empty: true }
    }

    fn close(&mut self, delimiter: char) -> Result<()> {
        self.ser.depth -= 1;
        if self.empty {
            write!(self.ser.w, "{delimiter}")?;
        } else {
            write!(
                self.ser.w,
                "\n{:indent$}{delimiter}",
                "",
                indent = self.ser.depth * 2
            )?;
        }
        Ok(())
    }
}

impl<W: Write> serde::ser::SerializeSeq for SerializeCollection<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.empty = false;
        write!(self.ser.w, "\n{:indent$}", "", indent = self.ser.depth * 2)?;
        value.serialize(&mut *self.ser)?;
        write!(self.ser.w, ",")?;
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok> {
        self.close(']')
    }
}

impl<W: Write> serde::ser::SerializeTuple for SerializeCollection<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<W: Write> serde::ser::SerializeTupleStruct for SerializeCollection<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<W: Write> serde::ser::SerializeTupleVariant for SerializeCollection<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(mut self) -> Result<Self::Ok> {
        self.close(']')?;
        self.ser.depth -= 1;
        write!(
            self.ser.w,
            ",\n{:indent$}}}",
            "",
            indent = self.ser.depth * 2
        )?;
        Ok(())
    }
}

impl<W: Write> serde::ser::SerializeMap for SerializeCollection<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.empty = false;
        write!(self.ser.w, "\n{:indent$}", "", indent = self.ser.depth * 2)?;
        key.serialize(MapKey::new(self.ser))?;
        write!(self.ser.w, ": ")?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.ser)?;
        write!(self.ser.w, ",")?;
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok> {
        self.close('}')
    }
}

impl<W: Write> serde::ser::SerializeStruct for SerializeCollection<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeMap::end(self)
    }
}

impl<W: Write> serde::ser::SerializeStructVariant for SerializeCollection<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(mut self) -> Result<Self::Ok> {
        self.close('}')?;
        self.ser.depth -= 1;
        write!(
            self.ser.w,
            ",\n{:indent$}}}",
            "",
            indent = self.ser.depth * 2
        )?;
        Ok(())
    }
}

macro_rules! serialize_quoted {
    ($method:ident, $type:ty) => {
        fn $method(self, v: $type) -> Result<Self::Ok> {
            write!(self.ser.w, "\"")?;
            self.ser.$method(v)?;
            write!(self.ser.w, "\"")?;
            Ok(())
        }
    };
}

struct MapKey<'a, W: Write> {
    ser: &'a mut Serializer<W>,
}

impl<'a, W: Write> MapKey<'a, W> {
    fn new(ser: &'a mut Serializer<W>) -> Self {
        Self { ser }
    }
}

impl<W: Write> serde::ser::Serializer for MapKey<'_, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    serialize_quoted!(serialize_u8, u8);
    serialize_quoted!(serialize_u16, u16);
    serialize_quoted!(serialize_u32, u32);
    serialize_quoted!(serialize_u64, u64);
    serialize_quoted!(serialize_i8, i8);
    serialize_quoted!(serialize_i16, i16);
    serialize_quoted!(serialize_i32, i32);
    serialize_quoted!(serialize_i64, i64);
    serialize_quoted!(serialize_f32, f32);
    serialize_quoted!(serialize_f64, f64);

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.ser.serialize_bool(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        if crate::char::is_json5_identifier_start(v) {
            write!(self.ser.w, "{v}")?;
        } else {
            self.ser.serialize_char(v)?;
        }
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let mut chars = v.chars();
        if let Some(first) = chars.next()
            && crate::char::is_json5_identifier_start(first)
            && chars.all(crate::char::is_json5_identifier)
        {
            write!(self.ser.w, "{v}")?;
        } else {
            self.ser.serialize_str(v)?;
        }
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.ser.serialize_bytes(v)
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.ser.serialize_some(v)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.ser.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        self.ser.serialize_unit_struct(name)
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::new(ErrorCode::InvalidKey))
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::new(ErrorCode::InvalidKey))
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::new(ErrorCode::InvalidKey))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        Err(Error::new(ErrorCode::InvalidKey))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::new(ErrorCode::InvalidKey))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::new(ErrorCode::InvalidKey))
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::new(ErrorCode::InvalidKey))
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        Err(Error::new(ErrorCode::InvalidKey))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::new(ErrorCode::InvalidKey))
    }
}
