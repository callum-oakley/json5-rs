extern crate pest;
#[macro_use]
extern crate pest_derive;

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
    pub fn from_str(_s: &str) -> Result<Value, Error<Rule>> {
        Ok(Value::Object(HashMap::new()))
    }
}
