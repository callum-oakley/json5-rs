use std::io::Write;

use serde::Serialize;

use crate::{Error, error::Result};

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    let mut w = Vec::new();
    to_writer(&mut w, value)?;
    Ok(String::from_utf8(w).expect("we only write valid UTF8"))
}

pub fn to_writer<T: Serialize, W: Write>(w: W, value: &T) -> Result<()> {
    value.serialize(&mut Serializer::new(w))
}

pub struct Serializer<W: Write> {
    w: W,
}

impl<W: Write> Serializer<W> {
    pub fn new(w: W) -> Self {
        Self { w }
    }
}

macro_rules! serialize_display {
    ($method:ident, $type:ident) => {
        fn $method(self, v: $type) -> Result<Self::Ok> {
            write!(self.w, "{v}").map_err(Into::into)
        }
    };
}

macro_rules! serialize_float {
    ($method:ident, $type:ident) => {
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

impl<W: Write> serde::ser::Serializer for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

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
        let delimeter = if v.contains('"') && !v.contains('\'') {
            '\''
        } else {
            '"'
        };
        write!(self.w, "{delimeter}")?;
        for c in v.chars() {
            match crate::char::escape(delimeter, c) {
                Some(escaped) => write!(self.w, "{escaped}")?,
                None => write!(self.w, "{c}")?,
            }
        }
        write!(self.w, "{delimeter}")?;
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
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        todo!()
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
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

impl<W: Write> serde::ser::SerializeSeq for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<W: Write> serde::ser::SerializeTuple for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<W: Write> serde::ser::SerializeTupleStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<W: Write> serde::ser::SerializeTupleVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<W: Write> serde::ser::SerializeMap for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<W: Write> serde::ser::SerializeStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<W: Write> serde::ser::SerializeStructVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}
