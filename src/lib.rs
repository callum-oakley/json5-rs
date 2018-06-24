extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
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
            Rule::boolean => match pair.as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                _ => unreachable!(),
            },
            Rule::string => Value::String(parse_string(pair.as_str())),
            Rule::number => Value::Number(parse_number(pair.as_str())),
            Rule::object => Value::Object(
                pair.into_inner()
                    .map(|member| {
                        let mut pairs = member.into_inner();
                        let key = parse_string(pairs.next().unwrap().as_str());
                        let value = Value::from_pair(pairs.next().unwrap());
                        (key, value)
                    })
                    .collect(),
            ),
            Rule::array => {
                Value::Array(pair.into_inner().map(Value::from_pair).collect())
            }
            _ => unreachable!(),
        }
    }
}

fn parse_string(s: &str) -> String {
    String::from(s.trim_matches(|c| c == '\'' || c == '"'))
}

fn parse_number(s: &str) -> f64 {
    if s.len() > 2 && &s[..2] == "0x" {
        i64::from_str_radix(&s[2..], 16).unwrap() as f64
    } else {
        s.parse().unwrap()
    }
}
