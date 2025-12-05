use std::collections::HashMap;

use json5::{Error, ErrorCode, Position, from_str};

use ErrorCode::*;
use serde_bytes::ByteBuf;
use serde_derive::Deserialize;
use serde_json::json;

fn err(code: ErrorCode) -> Error {
    Error::new(code)
}

fn err_at(line: usize, column: usize, code: ErrorCode) -> Error {
    Error::new_at(Position { line, column }, code)
}

fn custom_err_at(line: usize, column: usize, msg: &str) -> Error {
    Error::custom_at(Position { line, column }, msg)
}

// https://262.ecma-international.org/5.1/#sec-7.8.1
#[test]
fn parse_null() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Unit;

    assert_eq!(from_str("null"), Ok(()));
    assert_eq!(from_str("null"), Ok(Unit));
    assert_eq!(from_str::<()>("false"), Err(err_at(0, 0, ExpectedNull)));
    assert_eq!(from_str::<()>("nil"), Err(err_at(0, 1, ExpectedNull)));
    assert_eq!(from_str::<()>("0"), Err(err_at(0, 0, ExpectedNull)));
    assert_eq!(from_str::<()>("n"), Err(err(EofParsingNull)));
}

// https://262.ecma-international.org/5.1/#sec-7.8.2
#[test]
fn parse_bool() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Newtype(bool);

    assert_eq!(from_str("true"), Ok(true));
    assert_eq!(from_str("false"), Ok(false));
    assert_eq!(from_str("true"), Ok(Newtype(true)));
    assert_eq!(from_str::<bool>("null"), Err(err_at(0, 0, ExpectedBool)));
    assert_eq!(from_str::<bool>("yes"), Err(err_at(0, 0, ExpectedBool)));
    assert_eq!(from_str::<bool>("0"), Err(err_at(0, 0, ExpectedBool)));
    assert_eq!(from_str::<bool>("t"), Err(err(EofParsingBool)));
}

// https://spec.json5.org/#numbers
#[test]
fn parse_number() {
    assert_eq!(from_str("0"), Ok(0));
    assert!(from_str::<f64>("-0.").is_ok_and(|f| f == 0. && f.is_sign_negative()));
    assert_eq!(from_str("123"), Ok(123i8));
    assert_eq!(from_str("123"), Ok(123u8));
    assert_eq!(from_str("123"), Ok(123i16));
    assert_eq!(from_str("123"), Ok(123u16));
    assert_eq!(from_str("123"), Ok(123i32));
    assert_eq!(from_str("123"), Ok(123u32));
    assert_eq!(from_str("123"), Ok(123i64));
    assert_eq!(from_str("123"), Ok(123u64));
    assert_eq!(from_str("123"), Ok(123i128));
    assert_eq!(from_str("123"), Ok(123u128));
    assert_eq!(from_str("-123"), Ok(-123));
    assert_eq!(from_str("123.456"), Ok(123.456f32));
    assert_eq!(from_str("123.456"), Ok(123.456f64));
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
    assert_eq!(
        from_str("340282366920938463463374607431768211455"),
        Ok(u128::MAX)
    );
    assert_eq!(
        from_str("-170141183460469231731687303715884105728"),
        Ok(i128::MIN)
    );
    assert_eq!(
        from_str("0xffffffffffffffffffffffffffffffff"),
        Ok(u128::MAX)
    );
    assert_eq!(
        from_str("-0x80000000000000000000000000000000"),
        Ok(i128::MIN)
    );

    assert_eq!(from_str::<u32>("0x"), Err(err(EofParsingNumber)));
    assert_eq!(from_str::<u32>("0x!"), Err(err_at(0, 2, ExpectedNumber)));
    assert_eq!(from_str::<f64>("inf"), Err(err_at(0, 0, ExpectedNumber)));
    assert_eq!(
        from_str::<u64>("0x10000000000000000"), // u64::MAX + 1
        Err(custom_err_at(
            0,
            0,
            "invalid type: integer `18446744073709551616` as u128, expected u64"
        ))
    );
    assert_eq!(
        from_str::<u32>("007"),
        Err(err_at(0, 0, ErrorCode::LeadingZero))
    );
    assert_eq!(
        from_str::<u64>("18446744073709551616"), // u64::MAX + 1
        Err(custom_err_at(
            0,
            0,
            "invalid type: integer `18446744073709551616` as u128, expected u64"
        ))
    );
    assert_eq!(
        from_str::<i64>("0x8000000000000000"), // i64::MAX + 1
        Err(custom_err_at(
            0,
            0,
            "invalid value: integer `9223372036854775808`, expected i64"
        ))
    );
    assert_eq!(
        from_str::<i64>("-0x8000000000000001"), // i64::MIN - 1
        Err(custom_err_at(
            0,
            0,
            "invalid type: integer `-9223372036854775809` as i128, expected i64"
        ))
    );
    assert_eq!(
        from_str::<i64>("340282366920938463463374607431768211456"), // u128::MAX + 1
        Err(custom_err_at(
            0,
            0,
            "number too large to fit in target type"
        ))
    );
    assert_eq!(
        from_str::<i64>("-170141183460469231731687303715884105729"), // i128::MIN - 1
        Err(custom_err_at(
            0,
            0,
            "number too small to fit in target type"
        ))
    );
    assert_eq!(
        from_str::<u128>("0x100000000000000000000000000000000"), // u128::MAX + 1
        Err(err_at(0, 34, OverflowParsingNumber))
    );
    assert_eq!(
        from_str::<i128>("-0x80000000000000000000000000000001"), // i128::MIN - 1
        Err(custom_err_at(
            0,
            0,
            "out of range integral type conversion attempted"
        ))
    );
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
        Err(custom_err_at(
            0,
            0,
            "invalid value: string \"ab\", expected a character"
        ))
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
        Err(custom_err_at(
            0,
            0,
            "invalid type: string \"two\\nlines\", expected a borrowed string",
        )),
    );
    assert_eq!(
        from_str::<char>("'ab'"),
        Err(custom_err_at(
            0,
            0,
            "invalid value: string \"ab\", expected a character"
        ))
    );
}

// https://spec.json5.org/#arrays
#[test]
fn parse_array() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct TupleStruct(char, i32, bool);

    assert_eq!(from_str::<Vec<()>>("[]"), Ok(vec![]));
    assert_eq!(from_str("[null]"), Ok(vec![()]));
    assert_eq!(from_str("[1, 2, 3]"), Ok(vec![1, 2, 3]));
    assert_eq!(from_str("['a', 4, 'three']"), Ok(('a', 4, "three")));
    assert_eq!(from_str("['b', 5, false]"), Ok(TupleStruct('b', 5, false)));
    assert_eq!(
        from_str("[[1, 2], [3, 4]]"),
        Ok(vec![vec![1, 2], vec![3, 4]])
    );
    assert_eq!(
        from_str(
            "
            [
                'a',
                'b',
                'c', // Trailing comma!
            ]
            "
        ),
        Ok(vec!['a', 'b', 'c'])
    );

    assert_eq!(
        from_str::<Vec<i32>>("null"),
        Err(err_at(0, 0, ExpectedOpeningBracket))
    );
    assert_eq!(
        from_str::<(i32, i32)>("[1, 2, 3]"),
        Err(err_at(0, 7, ExpectedClosingBracket))
    );
    assert_eq!(
        from_str::<Vec<i32>>("[1, 2"),
        Err(err_at(0, 0, EofParsingArray))
    );
    assert_eq!(
        from_str::<Vec<i32>>("[1 2]"),
        Err(err_at(0, 3, ExpectedComma))
    );
    assert_eq!(
        from_str::<Vec<i32>>("[ , ]"),
        Err(err_at(0, 2, ExpectedNumber))
    );
}

// https://spec.json5.org/#objects
#[test]
fn parse_object() {
    #[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
    enum E {
        A,
        B,
        C(()),
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Example<'a> {
        #[serde(borrow)]
        image: Image<'a>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    struct Image<'a> {
        width: usize,
        height: usize,
        aspect_ratio: &'a str,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Person {
        name: String,
        age: u32,
    }

    assert_eq!(
        from_str::<HashMap<&str, usize>>(
            "
            // An empty object
            {}
            "
        ),
        Ok(HashMap::new())
    );
    assert_eq!(
        from_str(
            "
            // An object with two properties
            // and a trailing comma
            {
                width: 1920,
                height: 1080,
            }
            "
        ),
        Ok(HashMap::from([("width", 1920), ("height", 1080)]))
    );
    assert_eq!(
        from_str(
            "
            // Objects can be nested
            {
                image: {
                    width: 1920,
                    height: 1080,
                    'aspect-ratio': '16:9',
                }
            }
            "
        ),
        Ok(Example {
            image: Image {
                width: 1920,
                height: 1080,
                aspect_ratio: "16:9",
            }
        })
    );
    assert_eq!(
        from_str(
            "
            // An array of objects
            [
                { name: 'Joe', age: 27 },
                { name: 'Jane', age: 32 },
            ]
            "
        ),
        Ok(vec![
            Person {
                name: "Joe".to_owned(),
                age: 27
            },
            Person {
                name: "Jane".to_owned(),
                age: 32
            }
        ])
    );
    assert_eq!(
        from_str(r#"{ _123 : 42 }"#),
        Ok(HashMap::from([("_123", 42)]))
    );
    assert_eq!(
        from_str(r#"{ \u4f60\u597d: 42 }"#),
        Ok(HashMap::from([("你好".to_owned(), 42)]))
    );
    assert_eq!(
        from_str("{ '0': 'zero', '1': 'one' }"),
        Ok(HashMap::from([(0, "zero"), (1, "one")]))
    );
    assert_eq!(
        from_str("{ true: 'yes', false: 'no' }"),
        Ok(HashMap::from([(true, "yes"), (false, "no")]))
    );
    assert_eq!(
        from_str(r#"{ null: "not sure why you'd want this" }"#),
        Ok(HashMap::from([((), "not sure why you'd want this")]))
    );
    assert_eq!(
        from_str("{ a: 0, b: 1 }"),
        Ok(HashMap::from([(Some('a'), 0), (Some('b'), 1)]))
    );
    assert_eq!(
        from_str("{ A: 0, B: 1 }"),
        Ok(HashMap::from([(E::A, 0), (E::B, 1)]))
    );

    assert_eq!(
        from_str::<HashMap<String, u32>>(r#"{ foo\nbar: 42 }"#),
        Err(err_at(0, 6, InvalidEscapeSequence))
    );
    assert_eq!(
        from_str::<HashMap<String, u32>>(r#"{ \u4f60\u597d\u0021: 42 }"#),
        Err(err_at(0, 14, ExpectedIdentifier))
    );
    assert_eq!(
        from_str::<HashMap<String, u32>>(r#"{ 123: 42 }"#),
        Err(err_at(0, 2, ExpectedIdentifier))
    );
    assert_eq!(
        from_str::<Person>("{ name 'Joe', age 27 }"),
        Err(err_at(0, 7, ExpectedColon))
    );
    assert_eq!(
        from_str::<Person>("{ name: 'Joe' age: 27 }"),
        Err(err_at(0, 14, ExpectedComma))
    );
    assert_eq!(
        from_str::<Person>("{ name: 'Joe', age: 27"),
        Err(err_at(0, 0, EofParsingObject))
    );
    assert_eq!(
        from_str::<Person>("[ name: 'Joe', age: 27 ]"),
        Err(err_at(0, 0, ExpectedOpeningBrace))
    );
    assert_eq!(
        from_str::<HashMap<E, i32>>("{ A: 0, B: 1, C: 2 }"),
        Err(custom_err_at(
            0,
            14,
            "invalid type: unit variant, expected newtype variant"
        ))
    );
    assert_eq!(
        from_str::<HashMap<E, i32>>("{ A: 0, B: 1, { C: null }: 2 }"),
        Err(err_at(0, 14, ExpectedIdentifier))
    );
    assert_eq!(
        from_str::<HashMap<E, i32>>("{ A: 0, B: 1, D: 2 }"),
        Err(custom_err_at(
            0,
            14,
            "unknown variant `D`, expected one of `A`, `B`, `C`"
        ))
    );
}

#[test]
fn deserialize_option() {
    assert_eq!(from_str::<Option<bool>>("null"), Ok(None));
    assert_eq!(from_str::<Option<bool>>("true"), Ok(Some(true)));
}

// Examples from https://serde.rs/json.html
#[test]
fn deserialize_structs_and_enums() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct W {
        a: i32,
        b: i32,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct X(i32, i32);

    #[derive(Debug, PartialEq, Deserialize)]
    struct Y(i32);

    #[derive(Debug, PartialEq, Deserialize)]
    struct Z;

    #[derive(Debug, PartialEq, Deserialize)]
    enum E {
        W { a: i32, b: i32 },
        X(i32, i32),
        Y(i32),
        Z,
    }

    assert_eq!(from_str("{ a: 0, b: 0 }"), Ok(W { a: 0, b: 0 }));
    assert_eq!(from_str("[0, 0]"), Ok(X(0, 0)));
    assert_eq!(from_str("0"), Ok(Y(0)));
    assert_eq!(from_str("null"), Ok(Z));
    assert_eq!(from_str("{ W: { a: 0, b: 0 } }"), Ok(E::W { a: 0, b: 0 }));
    assert_eq!(from_str("{ X: [0, 0] }"), Ok(E::X(0, 0)));
    assert_eq!(from_str("{ Y: 0 }"), Ok(E::Y(0)));
    assert_eq!(from_str("'Z'"), Ok(E::Z));

    assert_eq!(
        from_str::<E>("'A'"),
        Err(custom_err_at(
            0,
            0,
            "unknown variant `A`, expected one of `W`, `X`, `Y`, `Z`"
        ))
    );
    assert_eq!(
        from_str::<E>("'W'"),
        Err(custom_err_at(
            0,
            0,
            "invalid type: unit variant, expected struct variant"
        ))
    );
    assert_eq!(
        from_str::<E>("{ W: 0 }"),
        Err(err_at(0, 5, ExpectedOpeningBrace))
    );
}

#[test]
fn comments() {
    assert_eq!(
        from_str(
            "
            // Single line comment
            /* Multi
               line
               comment */
            null
            "
        ),
        Ok(())
    );
}

#[test]
fn bytes() {
    assert_eq!(from_str("'4a534f4e35'"), Ok(ByteBuf::from("JSON5")));
    assert_eq!(from_str("'4A534F4E35'"), Ok(ByteBuf::from("JSON5")));
    assert_eq!(from_str("'4A534F4E35'"), Ok(ByteBuf::from("JSON5")));
    assert_eq!(
        from_str("{ '4a534f4e35': true }"),
        Ok(HashMap::from([(ByteBuf::from("JSON5"), true)])),
    );
    assert_eq!(
        from_str("{ '4A534F4E35': true }"),
        Ok(HashMap::from([(ByteBuf::from("JSON5"), true)])),
    );

    assert_eq!(
        from_str::<ByteBuf>("'4a534f4e3'"),
        Err(err_at(0, 0, InvalidBytes))
    );
    assert_eq!(
        from_str::<HashMap<ByteBuf, bool>>("{ '4a534f4e3g': true }"),
        Err(err_at(0, 2, InvalidBytes))
    );
    assert_eq!(
        from_str::<&[u8]>("'4a534f4e35'"),
        Err(custom_err_at(
            0,
            0,
            "invalid type: byte array, expected a borrowed byte array"
        ))
    );
}

#[test]
fn trailing_characters() {
    assert_eq!(
        from_str::<bool>("true false"),
        Err(err_at(0, 5, ErrorCode::TrailingCharacters))
    );
}

// "Kitchen-sink example" from https://json5.org/
#[test]
fn json5_org_example() {
    assert_eq!(
        from_str(
            r#"
            {
                // comments
                unquoted: 'and you can quote me on that',
                singleQuotes: 'I can use "double quotes" here',
                lineBreaks: "Look, Mom! \
No \\n's!",
                hexadecimal: 0xdecaf,
                leadingDecimalPoint: .8675309, andTrailing: 8675309.,
                positiveSign: +1,
                trailingComma: 'in objects', andIn: ['arrays',],
                "backwardsCompatible": "with JSON",
            }
            "#
        ),
        Ok(json!({
            "unquoted": "and you can quote me on that",
            "singleQuotes": "I can use \"double quotes\" here",
            "lineBreaks": "Look, Mom! No \\n's!",
            "hexadecimal": 0xdecaf,
            "leadingDecimalPoint": 0.8675309,
            "andTrailing": 8675309.0,
            "positiveSign": 1,
            "trailingComma": "in objects",
            "andIn": ["arrays"],
            "backwardsCompatible": "with JSON",
        }))
    )
}

// Example from
// https://github.com/chromium/chromium/blob/d1987ba98386e8a455c0b983650dfd9687d14636/third_party/blink/renderer/platform/runtime_enabled_features.json5.
// Suggested as a "more real-world example" on https://json5.org/.
#[test]
fn chromium_example() {
    // Just pull out the first value as a sanity check.
    assert_eq!(
        from_str::<serde_json::Value>(include_str!("chromium_example.json5"))
            .map(|config| config["parameters"]["status"]["valid_values"][0].clone()),
        Ok(serde_json::Value::String("stable".to_owned()))
    );
}
