extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::{Error, Parser};
use std::char;
use std::collections::HashMap;
use std::f64::{INFINITY, NAN, NEG_INFINITY};

const _GRAMMAR: &str = include_str!("json5.pest");

#[derive(Parser)]
#[grammar = "json5.pest"]
struct Json5Parser;

/// Represents any valid JSON5 value.
#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    String(String),
    Number(f64),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
}

impl Value {
    pub fn from_str(s: &str) -> Result<Value, Error<Rule>> {
        let mut pairs = Json5Parser::parse(Rule::text, s)?;

        Ok(Value::from_pair(pairs.next().unwrap()))
    }

    fn from_pair(pair: Pair<Rule>) -> Value {
        match pair.as_rule() {
            Rule::null => Value::Null,
            Rule::boolean => Value::Bool(parse_bool(pair)),
            Rule::string => Value::String(parse_string(pair)),
            Rule::number => Value::Number(parse_number(pair)),
            Rule::object => Value::Object(parse_object(pair)),
            Rule::array => Value::Array(parse_array(pair)),
            _ => unreachable!(),
        }
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
            Rule::character_literal => String::from(component.as_str()),
            Rule::character_escape_sequence => {
                parse_character_escape_sequence(component)
            }
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

fn parse_character_escape_sequence(pair: Pair<Rule>) -> String {
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

fn parse_object(pair: Pair<Rule>) -> HashMap<String, Value> {
    pair.into_inner()
        .map(|member| {
            let mut pairs = member.into_inner();
            let key = parse_string(pairs.next().unwrap());
            let value = Value::from_pair(pairs.next().unwrap());
            (key, value)
        })
        .collect()
}

fn parse_array(pair: Pair<Rule>) -> Vec<Value> {
    pair.into_inner().map(Value::from_pair).collect()
}

fn parse_hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}

fn is_hex_literal(s: &str) -> bool {
    s.len() > 2 && (&s[..2] == "0x" || &s[..2] == "0X")
}
