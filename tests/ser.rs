use json5::to_string;
use serde_bytes::{ByteBuf, Bytes};
use serde_derive::Serialize;

// https://spec.json5.org/#values
#[test]
fn serialize_null() {
    #[derive(Debug, PartialEq, Serialize)]
    struct A;

    assert_eq!(to_string(&()), Ok("null".to_owned()));
    assert_eq!(to_string(&A), Ok("null".to_owned()));
    assert_eq!(to_string::<Option<i32>>(&None), Ok("null".to_owned()));
}

// https://spec.json5.org/#values
#[test]
fn serialize_bool() {
    assert_eq!(to_string(&true), Ok("true".to_owned()));
    assert_eq!(to_string(&false), Ok("false".to_owned()));
    assert_eq!(to_string(&Some(true)), Ok("true".to_owned()));
}

// https://spec.json5.org/#numbers
#[test]
fn serialize_number() {
    assert_eq!(to_string(&123), Ok("123".to_owned()));
    assert_eq!(to_string(&123.456), Ok("123.456".to_owned()));
    assert_eq!(to_string(&-123.456), Ok("-123.456".to_owned()));
    assert_eq!(to_string(&f64::INFINITY), Ok("Infinity".to_owned()));
    assert_eq!(to_string(&-f64::INFINITY), Ok("-Infinity".to_owned()));
    assert_eq!(to_string(&f64::NAN), Ok("NaN".to_owned()));
    assert_eq!(to_string(&-f64::NAN), Ok("-NaN".to_owned()));
}

// https://spec.json5.org/#strings
#[test]
fn serialize_string() {
    assert_eq!(to_string(&"foo"), Ok(r#""foo""#.to_owned()));
    assert_eq!(to_string(&r#"double: ""#), Ok(r#"'double: "'"#.to_owned()));
    assert_eq!(to_string(&r"single: '"), Ok(r#""single: '""#.to_owned()));
    assert_eq!(
        to_string(&r#"double: ", single: '"#),
        Ok(r#""double: \", single: '""#.to_owned())
    );
    assert_eq!(
        to_string(&"escapes: \\, \n, \r, \u{2028}, \u{2029}"),
        Ok(r#""escapes: \\, \n, \r, \u2028, \u2029""#.to_owned())
    );
}

#[test]
fn serialize_bytes() {
    assert_eq!(
        to_string(&Bytes::new(&[0, 1, 2])),
        Ok(r#""000102""#.to_owned())
    );
    assert_eq!(
        to_string(&Bytes::new(b"JSON5")),
        Ok(r#""4a534f4e35""#.to_owned())
    );
    assert_eq!(
        to_string(&ByteBuf::from("JSON5")),
        Ok(r#""4a534f4e35""#.to_owned())
    );
    // TODO object keys
}

// https://spec.json5.org/#arrays
#[test]
fn serialize_array() {
    assert_eq!(to_string::<[i32; 0]>(&[]), Ok("[]".to_owned()));
    assert_eq!(
        to_string(&[0, 1, 2]),
        Ok("[\n  0,\n  1,\n  2,\n]".to_owned())
    );
    assert_eq!(
        to_string(&vec![vec![0], vec![1, 2]]),
        Ok("[\n  [\n    0,\n  ],\n  [\n    1,\n    2,\n  ],\n]".to_owned())
    );
    assert_eq!(
        to_string(&(1, true, "three")),
        Ok("[\n  1,\n  true,\n  \"three\",\n]".to_owned())
    );
}
