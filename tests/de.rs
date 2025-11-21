use std::f64;

use json5::{Error, ErrorCode, Position, from_str};

use ErrorCode::*;

fn err_at(line: usize, column: usize, code: ErrorCode) -> Error {
    Error::new_at(Position { line, column }, code)
}

fn err(code: ErrorCode) -> Error {
    Error::new(code)
}

// https://262.ecma-international.org/5.1/#sec-7.8.1
#[test]
fn parse_null() {
    assert_eq!(from_str("null"), Ok(()));
    assert_eq!(from_str::<()>("false"), Err(err_at(0, 0, ExpectedNull)));
    assert_eq!(from_str::<()>("nil"), Err(err_at(0, 1, ExpectedNull)));
    assert_eq!(from_str::<()>("0"), Err(err_at(0, 0, ExpectedNull)));
    assert_eq!(from_str::<()>("n"), Err(err(EofParsingNull)));
}

// https://262.ecma-international.org/5.1/#sec-7.8.2
#[test]
fn parse_bool() {
    assert_eq!(from_str("true"), Ok(true));
    assert_eq!(from_str("false"), Ok(false));
    assert_eq!(from_str::<bool>("null"), Err(err_at(0, 0, ExpectedBool)));
    assert_eq!(from_str::<bool>("yes"), Err(err_at(0, 0, ExpectedBool)));
    assert_eq!(from_str::<bool>("0"), Err(err_at(0, 0, ExpectedBool)));
    assert_eq!(from_str::<bool>("t"), Err(err(EofParsingBool)));
}

// https://spec.json5.org/#strings
#[test]
fn parse_string() {
    assert_eq!(from_str(r#""can borrow""#), Ok("can borrow"));
    assert_eq!(from_str(r#"'"quotes"'"#), Ok(r#""quotes""#));
    assert_eq!(from_str(r#""'quotes'""#), Ok("'quotes'"));
    assert_eq!(from_str(r#"'你好!'"#), Ok("你好!"));
    assert_eq!(from_str("'\u{2028}\u{2029}'"), Ok("\u{2028}\u{2029}"));
    assert_eq!(from_str(r#"'a'"#), Ok('a'));
    assert_eq!(from_str(r#"'好'"#), Ok('好'));
    assert_eq!(from_str(r#""two\nlines""#), Ok("two\nlines".to_owned()));
    assert_eq!(from_str("'one \\\nline'"), Ok("one line".to_owned()));
    assert_eq!(from_str("'one \\\r\nline'"), Ok("one line".to_owned()));
    assert_eq!(from_str(r#""zero: '\0'""#), Ok("zero: '\0'".to_owned()));
    assert_eq!(from_str(r#"'\u4f60\u597d\x21'"#), Ok("你好!".to_owned()));

    assert_eq!(
        from_str::<char>(r#"'ab'"#),
        Err(err(Message(
            "invalid value: string \"ab\", expected a character".to_owned()
        )))
    );
    assert_eq!(
        from_str::<String>("'one\ntwo'"),
        Err(err_at(0, 4, LineTerminatorInString))
    );
    assert_eq!(
        from_str::<String>(r#"'\01'"#),
        Err(err_at(0, 3, InvalidEscapeSequence))
    );
    assert_eq!(
        from_str::<String>(r#"'\42'"#),
        Err(err_at(0, 2, InvalidEscapeSequence))
    );
    assert_eq!(
        from_str::<String>(r#"'\ubar'"#),
        Err(err_at(0, 5, InvalidEscapeSequence))
    );
    assert_eq!(
        from_str::<String>("false"),
        Err(err_at(0, 0, ExpectedString))
    );
    assert_eq!(from_str::<String>("'..."), Err(err(EofParsingString)));
    assert_eq!(
        from_str::<&str>(r#""two\nlines""#),
        Err(err(Message(
            "invalid type: string \"two\\nlines\", expected a borrowed string".to_owned(),
        ))),
    );
}

// https://spec.json5.org/#numbers
#[test]
fn parse_number() {
    assert_eq!(from_str("0"), Ok(0));
    assert!(from_str::<f64>("-0.").is_ok_and(|f| f == 0. && f.is_sign_negative()));
    assert_eq!(from_str("123"), Ok(123i32));
    assert_eq!(from_str("123"), Ok(123u32));
    assert_eq!(from_str("-123"), Ok(-123));
    assert_eq!(from_str("123.456"), Ok(123.456));
    assert_eq!(from_str("123.0"), Ok(123.));
    assert_eq!(from_str("123."), Ok(123.));
    assert_eq!(from_str(".456"), Ok(0.456));
    assert_eq!(from_str("0.456"), Ok(0.456));
    assert_eq!(from_str("123e-456"), Ok(123e-456));
    assert_eq!(from_str("123E-456"), Ok(123e-456));
    assert_eq!(from_str("18446744073709551615"), Ok(u64::MAX));
    assert_eq!(from_str("Infinity"), Ok(f64::INFINITY));
    assert_eq!(from_str("-Infinity"), Ok(-f64::INFINITY));
    assert!(from_str::<f64>("NaN").is_ok_and(|f| f.is_nan() && f.is_sign_positive()));
    assert!(from_str::<f64>("-NaN").is_ok_and(|f| f.is_nan() && f.is_sign_negative()));
    assert_eq!(from_str("0xdecaf"), Ok(0xdecaf));
    assert_eq!(from_str("-0xC0FFEE"), Ok(-0xC0FFEE));
    assert_eq!(from_str("0x7FFFFFFFFFFFFFFF"), Ok(i64::MAX));
    assert_eq!(from_str("-0x8000000000000000"), Ok(i64::MIN));
    assert_eq!(from_str("0x0"), Ok(0));

    assert_eq!(from_str::<u32>("0x"), Err(err(EofParsingNumber)));
    assert_eq!(from_str::<u32>("0x!"), Err(err_at(0, 2, ExpectedNumber)));
    assert_eq!(from_str::<f64>("inf"), Err(err_at(0, 0, ExpectedNumber)));
    assert_eq!(
        from_str::<u64>("0x10000000000000000"), // u64::MAX + 1
        Err(err_at(0, 18, OverflowParsingNumber))
    );
    assert_eq!(
        from_str::<u32>("007"),
        Err(err_at(0, 0, ErrorCode::LeadingZero))
    );
    assert_eq!(
        from_str::<u64>("18446744073709551616"), // u64::MAX + 1
        Err(err_at(
            0,
            0,
            Message("number too large to fit in target type".to_owned())
        ))
    );
    assert_eq!(
        from_str::<i64>("0x8000000000000000"), // i64::MAX + 1
        Err(err(Message(
            "invalid value: integer `9223372036854775808`, expected i64".to_owned()
        )))
    );
    assert_eq!(
        from_str::<i64>("-0x8000000000000001"), // i64::MIN - 1
        Err(err_at(
            0,
            0,
            Message("out of range integral type conversion attempted".to_owned())
        ))
    );
}

#[test]
fn deserialize_option() {
    assert_eq!(from_str::<Option<bool>>("null"), Ok(None));
    assert_eq!(from_str::<Option<bool>>("true"), Ok(Some(true)));
}
