use pest::iterators::{Pair, Pairs};
use pest::Parser;
use serde::de::{
    Deserialize, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor,
};
use std::char;
#[cfg(test)]
use std::collections::HashMap;
use std::f64::{INFINITY, NAN, NEG_INFINITY};

use error::{Error, Result};

const _GRAMMAR: &str = include_str!("json5.pest");

#[derive(Parser)]
#[grammar = "json5.pest"]
struct JSON5Parser;

pub struct Json5Deserializer<'de> {
    pair: Option<Pair<'de, Rule>>,
}

impl<'de> Json5Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Result<Self> {
        let pair = JSON5Parser::parse(Rule::text, input)?.next().unwrap();
        Ok(Json5Deserializer::from_pair(pair))
    }

    fn from_pair(pair: Pair<'de, Rule>) -> Self {
        Json5Deserializer { pair: Some(pair) }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Json5Deserializer::from_str(s)?;
    T::deserialize(&mut deserializer)
}

impl<'de, 'a> Deserializer<'de> for &'a mut Json5Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        match pair.as_rule() {
            Rule::null => visitor.visit_unit(),
            Rule::boolean => visitor.visit_bool(parse_bool(pair)),
            Rule::string | Rule::identifier => {
                visitor.visit_string(parse_string(pair))
            }
            Rule::number => visitor.visit_f64(parse_number(pair)),
            Rule::array => visitor.visit_seq(Access::to(pair.into_inner())),
            Rule::object => visitor.visit_map(Access::to(pair.into_inner())),
            _ => unreachable!(),
        }
    }

    // The below will get us the right types, but won't necessarily give
    // meaningful results if the source is out of the range of the target type.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_i8(parse_number(pair) as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_i16(parse_number(pair) as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_i32(parse_number(pair) as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_i64(parse_number(pair) as i64)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_i128(parse_number(pair) as i128)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_u8(parse_number(pair) as u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_u16(parse_number(pair) as u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_u32(parse_number(pair) as u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_u64(parse_number(pair) as u64)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_u128(parse_number(pair) as u128)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_f32(parse_number(pair) as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = self.pair.take().unwrap();
        visitor.visit_f64(parse_number(pair))
    }

    // TODO Probably don't want to forward enum, struct, etc...
    forward_to_deserialize_any! {
        bool char str string bytes byte_buf option unit unit_struct
        newtype_struct seq tuple tuple_struct map struct enum identifier
        ignored_any
    }
}

fn parse_bool(pair: Pair<Rule>) -> bool {
    match pair.as_str() {
        "true" => true,
        "false" => false,
        _ => unreachable!(),
    }
}

fn parse_string(pair: Pair<Rule>) -> String {
    pair.into_inner()
        .map(|component| match component.as_rule() {
            Rule::char_literal => String::from(component.as_str()),
            Rule::char_escape_sequence => parse_char_escape_sequence(component),
            Rule::nul_escape_sequence => String::from("\u{0000}"),
            Rule::hex_escape_sequence | Rule::unicode_escape_sequence => {
                char::from_u32(parse_hex(component.as_str()))
                    .unwrap()
                    .to_string()
            }
            _ => unreachable!(),
        })
        .collect()
}

fn parse_char_escape_sequence(pair: Pair<Rule>) -> String {
    String::from(match pair.as_str() {
        "b" => "\u{0008}",
        "f" => "\u{000C}",
        "n" => "\n",
        "r" => "\r",
        "t" => "\t",
        "v" => "\u{000B}",
        c => c,
    })
}

fn parse_number(pair: Pair<Rule>) -> f64 {
    match pair.as_str() {
        "Infinity" => INFINITY,
        "-Infinity" => NEG_INFINITY,
        "NaN" | "-NaN" => NAN,
        s if is_hex_literal(s) => parse_hex(&s[2..]) as f64,
        s => s.parse().unwrap(),
    }
}

fn parse_hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}

fn is_hex_literal(s: &str) -> bool {
    s.len() > 2 && (&s[..2] == "0x" || &s[..2] == "0X")
}

struct Access<'de> {
    pairs: Pairs<'de, Rule>,
}

impl<'de> Access<'de> {
    fn to(pairs: Pairs<'de, Rule>) -> Self {
        Access { pairs }
    }
}

impl<'de> SeqAccess<'de> for Access<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(pair) = self.pairs.next() {
            seed.deserialize(&mut Json5Deserializer::from_pair(pair))
                .map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<'de> MapAccess<'de> for Access<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some(pair) = self.pairs.next() {
            seed.deserialize(&mut Json5Deserializer::from_pair(pair))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(pair) = self.pairs.next() {
            seed.deserialize(&mut Json5Deserializer::from_pair(pair))
        } else {
            unreachable!()
        }
    }
}

#[test]
fn test_null() {
    assert_eq!(from_str("null"), Ok(()));
}

#[test]
fn test_bool() {
    assert_eq!(from_str("true"), Ok(true));
    assert_eq!(from_str("false"), Ok(false));
}

#[test]
fn test_string() {
    assert_eq!(from_str("\"true\""), Ok(String::from("true")));
    assert_eq!(
        from_str("'a string! with a double quote (\") in it'"),
        Ok(String::from("a string! with a double quote (\") in it"))
    );
}

#[test]
fn test_number() {
    assert_eq!(from_str("0x00000F"), Ok(15))
}

#[test]
fn test_array() {
    assert_eq!(
        from_str("[[1, 2], [3], []]"),
        Ok(vec![vec![1, 2], vec![3], vec![]])
    )
}

#[test]
fn test_object() {
    let mut m = HashMap::new();
    m.insert(String::from("one"), 1);
    m.insert(String::from("two"), 2);
    assert_eq!(from_str("{ one: 1, two: 2 }"), Ok(m))
}
