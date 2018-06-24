extern crate json5_parser;

use json5_parser::Value as V;
use std::collections::HashMap;
use std::f64::{INFINITY, NEG_INFINITY};

fn parses_to(s: &str, v: V) {
    assert_eq!(V::from_str(s), Ok(v));
}

fn parses_to_nan(s: &str) {
    if let V::Number(n) = V::from_str(s).unwrap() {
        assert_eq!(n.is_nan(), true)
    } else {
        panic!("expected Value::Number")
    }
}

// The following tests are adapted from https://github.com/json5/json5/blob/d828908384ce8dc40d8dde017ae82afd1b952d79/test/parse.js

// objects

#[test]
fn parses_empty_objects() {
    parses_to("{}", V::Object(HashMap::new()));
}

#[test]
fn parses_double_string_property_names() {
    let mut m = HashMap::new();
    m.insert(String::from("a"), V::Number(1.));
    parses_to("{\"a\":1}", V::Object(m));
}

#[test]
fn parses_single_string_property_names() {
    let mut m = HashMap::new();
    m.insert(String::from("a"), V::Number(1.));
    parses_to("{'a':1}", V::Object(m));
}

#[test]
fn parses_unquoted_property_names() {
    let mut m = HashMap::new();
    m.insert(String::from("a"), V::Number(1.));
    parses_to("{a:1}", V::Object(m));
}

#[test]
fn parses_special_character_property_names() {
    let mut m = HashMap::new();
    m.insert(String::from("$_"), V::Number(1.));
    m.insert(String::from("_$"), V::Number(2.));
    m.insert(String::from("a\u{200C}"), V::Number(3.));
    parses_to("{$_:1,_$:2,a\u{200C}:3}", V::Object(m));
}

#[test]
fn parses_unicode_property_names() {
    let mut m = HashMap::new();
    m.insert(String::from("ùńîċõďë"), V::Number(9.));
    parses_to("{ùńîċõďë:9}", V::Object(m));
}

#[test]
fn parses_escaped_property_names() {
    let mut m = HashMap::new();
    m.insert(String::from("ab"), V::Number(1.));
    m.insert(String::from("$_"), V::Number(2.));
    m.insert(String::from("_$"), V::Number(3.));
    parses_to(
        "{\\u0061\\u0062:1,\\u0024\\u005F:2,\\u005F\\u0024:3}",
        V::Object(m),
    );
}

#[test]
fn parses_multiple_properties() {
    let mut m = HashMap::new();
    m.insert(String::from("abc"), V::Number(1.));
    m.insert(String::from("def"), V::Number(2.));
    parses_to("{abc:1,def:2}", V::Object(m));
}

#[test]
fn parses_nested_objects() {
    let mut inner = HashMap::new();
    inner.insert(String::from("b"), V::Number(2.));
    let mut outer = HashMap::new();
    outer.insert(String::from("a"), V::Object(inner));
    parses_to("{a:{b:2}}", V::Object(outer));
}

// arrays

#[test]
fn parses_empty_arrays() {
    parses_to("[]", V::Array(vec![]));
}

#[test]
fn parses_array_values() {
    parses_to("[1]", V::Array(vec![V::Number(1.)]));
}

#[test]
fn parses_multiple_array_values() {
    parses_to("[1,2]", V::Array(vec![V::Number(1.), V::Number(2.)]));
}

#[test]
fn parses_nested_arrays() {
    parses_to(
        "[1,[2,3]]",
        V::Array(vec![
            V::Number(1.),
            V::Array(vec![V::Number(2.), V::Number(3.)]),
        ]),
    );
}

#[test]
fn parses_nulls() {
    parses_to("null", V::Null);
}

#[test]
fn parses_true() {
    parses_to("true", V::Bool(true));
}

#[test]
fn parses_false() {
    parses_to("false", V::Bool(false));
}

// numbers

#[test]
fn parses_leading_zeroes() {
    parses_to(
        "[0,0.,0e0]",
        V::Array(vec![V::Number(0.), V::Number(0.), V::Number(0.)]),
    );
}

#[test]
fn parses_integers() {
    parses_to(
        "[1,23,456,7890]",
        V::Array(vec![
            V::Number(1.),
            V::Number(23.),
            V::Number(456.),
            V::Number(7890.),
        ]),
    );
}

#[test]
fn parses_signed_numbers() {
    parses_to(
        "[-1,+2,-.1,-0]",
        V::Array(vec![
            V::Number(-1.),
            V::Number(2.),
            V::Number(-0.1),
            V::Number(-0.),
        ]),
    );
}

#[test]
fn parses_leading_decimal_points() {
    parses_to("[.1,.23]", V::Array(vec![V::Number(0.1), V::Number(0.23)]));
}

#[test]
fn parses_fractional_numbers() {
    parses_to("[1.0,1.23]", V::Array(vec![V::Number(1.), V::Number(1.23)]));
}

#[test]
fn parses_exponents() {
    parses_to(
        "[1e0,1e1,1e01,1.e0,1.1e0,1e-1,1e+1]",
        V::Array(vec![
            V::Number(1.),
            V::Number(10.),
            V::Number(10.),
            V::Number(1.),
            V::Number(1.1),
            V::Number(0.1),
            V::Number(10.),
        ]),
    );
}

#[test]
fn parses_hexadecimal_numbers() {
    parses_to(
        "[0x1,0x10,0xff,0xFF]",
        V::Array(vec![
            V::Number(1.),
            V::Number(16.),
            V::Number(255.),
            V::Number(255.),
        ]),
    );
}

#[test]
fn parses_signed_and_unsiged_infinity() {
    parses_to(
        "[Infinity,-Infinity]",
        V::Array(vec![V::Number(INFINITY), V::Number(NEG_INFINITY)]),
    );
}

#[test]
fn parses_signed_and_unsigned_nan() {
    parses_to_nan("NaN");
    parses_to_nan("-NaN");
}

// strings

#[test]
fn parses_double_quoted_strings() {
    parses_to("\"abc\"", V::String(String::from("abc")));
}

#[test]
fn parses_single_quoted_strings() {
    parses_to("'abc'", V::String(String::from("abc")));
}

#[test]
fn parses_nested_quotes_strings() {
    parses_to(
        "['\"',\"'\"]",
        V::Array(vec![
            V::String(String::from("\"")),
            V::String(String::from("'")),
        ]),
    );
}

#[test]
fn parses_escaped_characters() {
    parses_to("'\\b\\f\\n\\r\\t\\v\\0\\x0f\\u01fF\\\n\\\r\n\\\r\\\u{2028}\\\u{2029}\\a\\'\\\"'", V::String(String::from("\u{0008}\u{000C}\n\r\t\u{000B}\0\x0f\u{01FF}a'\"")));
}

// comments

#[test]
fn parses_single_line_comments() {
    parses_to("{//comment\n}", V::Object(HashMap::new()));
}

#[test]
fn parses_single_line_comments_at_end_of_input() {
    parses_to("{}//comment", V::Object(HashMap::new()));
}

#[test]
fn parses_multi_line_comments() {
    parses_to("{/*comment\n** */}", V::Object(HashMap::new()));
}

#[test]
fn parses_whitespace() {
    parses_to(
        "{\t\u{000B}\u{000C} \u{00A0}\u{FEFF}\n\r\u{2028}\u{2029}\u{2003}}",
        V::Object(HashMap::new()),
    );
}
