extern crate pest;
#[macro_use]
extern crate pest_derive;

const _GRAMMAR: &str = include_str!("json5.pest");

#[derive(Parser)]
#[grammar = "json5.pest"]
pub struct Json5Parser;
