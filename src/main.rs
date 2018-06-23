extern crate json5_parser;
extern crate pest;

use json5_parser::{Json5Parser, Rule};
use pest::Parser;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut unparsed_file = String::new();
    File::open("test.json")
        .expect("cannot open file")
        .read_to_string(&mut unparsed_file)
        .expect("cannot read file");

    println!("{:?}", Json5Parser::parse(Rule::json, &unparsed_file))
}
