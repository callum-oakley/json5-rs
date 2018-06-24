extern crate json5_parser;

use json5_parser::Value as V;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn parses_example_from_json5_dot_org() {
    let mut contents = String::new();
    File::open("tests/assets/json5_dot_org_example.json5")
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    let m = vec![
        (
            "unquoted",
            V::String(String::from("and you can quote me on that")),
        ),
        (
            "singleQuotes",
            V::String(String::from("I can use \"double quotes\" here")),
        ),
        (
            "lineBreaks",
            V::String(String::from("Look, Mom! No \\n's!")),
        ),
        ("hexadecimal", V::Number(0xdecaf as f64)),
        ("leadingDecimalPoint", V::Number(0.8675309)),
        ("andTrailing", V::Number(8675309.)),
        ("positiveSign", V::Number(1.)),
        ("trailingComma", V::String(String::from("in objects"))),
        ("andIn", V::Array(vec![V::String(String::from("arrays"))])),
        ("backwardsCompatible", V::String(String::from("with JSON"))),
    ].into_iter()
        .map(|(k, v)| (String::from(k), v))
        .collect();

    assert_eq!(V::from_str(&contents), Ok(V::Object(m)),);
}
