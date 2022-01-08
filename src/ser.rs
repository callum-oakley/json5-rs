use serde::ser::{self, Serialize};
use std::{f32, f64};

use crate::error::{Error, Result};

/// Attempts to serialize the input as a JSON5 string (actually a JSON string).
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

struct Serializer {
    output: String,
    // TODO settings for formatting (single vs double quotes, whitespace etc)
}

impl Serializer {
    fn call_to_string<T>(&mut self, v: &T) -> Result<()>
    where
        T: ToString,
    {
        self.output += &v.to_string();
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.call_to_string(&v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        if v == f32::INFINITY {
            self.output += "Infinity";
        } else if v == f32::NEG_INFINITY {
            self.output += "-Infinity";
        } else if v.is_nan() {
            self.output += "NaN";
        } else {
            self.call_to_string(&v)?;
        }
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        if v == f64::INFINITY {
            self.output += "Infinity";
        } else if v == f64::NEG_INFINITY {
            self.output += "-Infinity";
        } else if v.is_nan() {
            self.output += "NaN";
        } else {
            self.call_to_string(&v)?;
        }
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += "\"";
        self.output += &escape(v);
        self.output += "\"";
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        unimplemented!() // TODO
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.output += "null";
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output += "{";
        variant.serialize(&mut *self)?; // TODO drop the quotes where possible
        self.output += ":";
        value.serialize(&mut *self)?;
        self.output += "}";
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output += "[";
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":[";
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output += "{";
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":{";
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        self.output += "]}";
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(KeySerializer { base: &mut **self })
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "}";
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<()> {
        self.output += "}}";
        Ok(())
    }
}

fn escape(v: &str) -> String {
    v.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', c],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            '/' => vec!['\\', '/'],
            '\\' => vec!['\\', '\\'],
            '\u{0008}' => vec!['\\', 'b'],
            '\u{000c}' => vec!['\\', 'f'],
            c => vec![c],
        })
        .collect()
}

struct KeySerializer<'a> {
    base: &'a mut Serializer,
}

impl KeySerializer<'_> {
    fn serialize_integer<I: itoa::Integer>(self, integer: I) -> Result<()> {
        self.base.output.push('"');
        self.base
            .output
            .push_str(itoa::Buffer::new().format(integer));
        self.base.output.push('"');
        Ok(())
    }
}

fn key_not_string() -> Error {
    ser::Error::custom("key must be a string")
}

impl ser::Serializer for KeySerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_str(self, v: &str) -> Result<()> {
        self.base.serialize_str(v)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_integer(v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_integer(v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_integer(v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_integer(v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_integer(v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_integer(v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_integer(v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_integer(v)
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(v.encode_utf8(&mut [0; 4]))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_unit(self) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_none(self) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<()> {
        Err(key_not_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_not_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_not_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_not_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_not_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_not_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_not_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_not_string())
    }

    fn collect_str<T: ?Sized + std::fmt::Display>(self, value: &T) -> Result<()> {
        self.base.collect_str(value)
    }
}
