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
        match pair.as_rule() {
            Rule::null => Value::Null,
            Rule::boolean => match pair.as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                _ => unreachable!(),
            },
            Rule::string => {
                Value::String(String::from(pair.as_str())) // TODO strip ' and "
            }
            Rule::number => Value::Number(pair.as_str().parse().unwrap()),
            Rule::object => Value::Object(
                pair.into_inner()
                    .map(|member| {
                        println!("MEMBER: {:?}", member);
                        let mut pairs = member.into_inner();
                        let key = String::from(pairs.next().unwrap().as_str());
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
