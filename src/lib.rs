extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::{Pair, Pairs};
use pest::{Error, Parser};
use std::collections::HashMap;

const _GRAMMAR: &str = include_str!("json5.pest");

#[derive(Parser)]
#[grammar = "json5.pest"]
struct Json5Parser;

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
        println!("from_pair:\n\t{:?}", pair);
        match pair.as_rule() {
            Rule::null => Value::Null,
            Rule::boolean => Value::Bool(parse_bool(pair.as_str())),
            Rule::string => Value::String(parse_string(pair.as_str())),
            Rule::number => Value::Number(parse_number(pair.as_str())),
            Rule::object => Value::Object(parse_object(pair.into_inner())),
            Rule::array => Value::Array(parse_array(pair.into_inner())),
            _ => unreachable!(),
        }
    }
}

fn parse_bool(s: &str) -> bool {
    match s {
        "true" => true,
        "false" => false,
        _ => unreachable!(),
    }
}

fn parse_string(s: &str) -> String {
    if &s[0..1] == "\"" || &s[0..1] == "'" {
        String::from(&s[1..s.len() - 1])
    } else {
        String::from(s)
    }
}

fn parse_number(s: &str) -> f64 {
    if s.len() > 2 && &s[..2] == "0x" {
        i64::from_str_radix(&s[2..], 16).unwrap() as f64
    } else {
        s.parse().unwrap()
    }
}

fn parse_object(members: Pairs<Rule>) -> HashMap<String, Value> {
    members
        .map(|member| {
            let mut pairs = member.into_inner();
            let key = parse_string(pairs.next().unwrap().as_str());
            let value = Value::from_pair(pairs.next().unwrap());
            (key, value)
        })
        .collect()
}

fn parse_array(elements: Pairs<Rule>) -> Vec<Value> {
    elements.map(Value::from_pair).collect()
}
