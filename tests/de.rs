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

    assert_eq!(from_str(r#""two\nlines""#), Ok("two\nlines".to_owned()));
    assert_eq!(from_str("'one \\\nline'"), Ok("one line".to_owned()));
    assert_eq!(from_str("'one \\\r\nline'"), Ok("one line".to_owned()));
    assert_eq!(from_str(r#""zero: '\0'""#), Ok("zero: '\0'".to_owned()));
    assert_eq!(from_str(r#"'\u4f60\u597d\x21'"#), Ok("你好!".to_owned()));

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
            r#"invalid type: string "two\nlines", expected a borrowed string"#.to_owned(),
        ))),
    );
}

// https://spec.json5.org/#numbers
#[test]
fn parse_number() {
    assert_eq!(from_str("42"), Ok(42));
}

#[test]
fn deserialize_option() {
    assert_eq!(from_str::<Option<bool>>("null"), Ok(None));
    assert_eq!(from_str::<Option<bool>>("true"), Ok(Some(true)));
}
