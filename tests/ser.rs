use indexmap::IndexMap;
use json5::{Error, ErrorCode, to_string};
use serde_bytes::{ByteBuf, Bytes};
use serde_derive::Serialize;

// https://spec.json5.org/#values
#[test]
fn serialize_null() {
    #[derive(Debug, PartialEq, Serialize)]
    struct A;

    assert_eq!(to_string(&()), Ok("null".to_owned()));
    assert_eq!(to_string(&A), Ok("null".to_owned()));
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
    assert_eq!(to_string(&123u8), Ok("123".to_owned()));
    assert_eq!(to_string(&123i8), Ok("123".to_owned()));
    assert_eq!(to_string(&123u16), Ok("123".to_owned()));
    assert_eq!(to_string(&123i16), Ok("123".to_owned()));
    assert_eq!(to_string(&123u32), Ok("123".to_owned()));
    assert_eq!(to_string(&123i32), Ok("123".to_owned()));
    assert_eq!(to_string(&123u64), Ok("123".to_owned()));
    assert_eq!(to_string(&123i64), Ok("123".to_owned()));
    assert_eq!(to_string(&123u128), Ok("123".to_owned()));
    assert_eq!(to_string(&123i128), Ok("123".to_owned()));
    assert_eq!(to_string(&123.456f32), Ok("123.456".to_owned()));
    assert_eq!(to_string(&-123.456f32), Ok("-123.456".to_owned()));
    assert_eq!(to_string(&123.456f64), Ok("123.456".to_owned()));
    assert_eq!(to_string(&-123.456f64), Ok("-123.456".to_owned()));
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

// https://spec.json5.org/#objects
#[test]
fn serialize_object() {
    #[derive(PartialEq, Eq, Hash, Serialize)]
    enum E {
        A,
        B,
        C(()),
    }

    #[derive(Serialize)]
    #[serde(rename_all = "kebab-case")]
    struct Image<'a> {
        width: usize,
        height: usize,
        aspect_ratio: &'a str,
    }

    assert_eq!(
        to_string::<IndexMap<&str, i32>>(&IndexMap::new()),
        Ok("{}".to_owned())
    );
    assert_eq!(
        to_string(&IndexMap::from([("foo", 0), ("bar", 1), ("a b", 3)])),
        Ok("{\n  foo: 0,\n  bar: 1,\n  \"a b\": 3,\n}".to_owned())
    );
    assert_eq!(
        to_string(&IndexMap::from([(ByteBuf::from("JSON5"), 0)])),
        Ok("{\n  \"4a534f4e35\": 0,\n}".to_owned())
    );
    assert_eq!(
        to_string(&IndexMap::from([(true, "yes"), (false, "no")])),
        Ok("{\n  true: \"yes\",\n  false: \"no\",\n}".to_owned())
    );
    assert_eq!(
        to_string(&IndexMap::from([
            ('τ', std::f64::consts::TAU),
            ('∞', f64::INFINITY),
        ])),
        Ok("{\n  τ: 6.283185307179586,\n  \"∞\": Infinity,\n}".to_owned())
    );
    assert_eq!(
        to_string(&IndexMap::from([(E::A, 'a'), (E::B, 'b'),])),
        Ok("{\n  A: \"a\",\n  B: \"b\",\n}".to_owned())
    );
    assert_eq!(
        to_string(&Image {
            width: 1920,
            height: 1080,
            aspect_ratio: "16:9",
        }),
        Ok("{\n  width: 1920,\n  height: 1080,\n  \"aspect-ratio\": \"16:9\",\n}".to_owned())
    );
    assert_eq!(
        to_string(&IndexMap::from([(0, "zero"), (1, "one")])),
        Ok("{\n  \"0\": \"zero\",\n  \"1\": \"one\",\n}".to_owned())
    );

    assert_eq!(
        to_string(&IndexMap::from([(E::A, 'a'), (E::B, 'b'), (E::C(()), 'c')])),
        Err(Error::new(ErrorCode::InvalidKey)),
    );
}

#[test]
fn serialize_option() {
    assert_eq!(to_string::<Option<i32>>(&None), Ok("null".to_owned()));
    assert_eq!(to_string::<Option<i32>>(&Some(42)), Ok("42".to_owned()));
}

#[test]
// Examples from https://serde.rs/json.html
fn serialize_structs_and_enums() {
    #[derive(Serialize)]
    struct W {
        a: i32,
        b: i32,
    }

    #[derive(Serialize)]
    struct X(i32, i32);

    #[derive(Serialize)]
    struct Y(i32);

    #[derive(Serialize)]
    struct Z;

    #[derive(Serialize)]
    enum E {
        W { a: i32, b: i32 },
        X(i32, i32),
        Y(i32),
        Z,
    }

    assert_eq!(
        to_string(&W { a: 0, b: 0 }),
        Ok("{\n  a: 0,\n  b: 0,\n}".to_owned())
    );
    assert_eq!(to_string(&X(0, 0)), Ok("[\n  0,\n  0,\n]".to_owned()));
    assert_eq!(to_string(&Y(0)), Ok("0".to_owned()));
    assert_eq!(to_string(&Z), Ok("null".to_owned()));
    assert_eq!(
        to_string(&E::W { a: 0, b: 0 }),
        Ok("{\n  W: {\n    a: 0,\n    b: 0,\n  },\n}".to_owned())
    );
    assert_eq!(
        to_string(&E::X(0, 0)),
        Ok("{\n  X: [\n    0,\n    0,\n  ],\n}".to_owned())
    );
    assert_eq!(to_string(&E::Y(0)), Ok("{\n  Y: 0,\n}".to_owned()));
    assert_eq!(to_string(&E::Z), Ok("\"Z\"".to_owned()));
}

#[test]
// https://serde.rs/enum-representations.html
fn enum_representations() {
    // https://serde.rs/enum-representations.html#externally-tagged
    {
        #[derive(Serialize)]
        enum Message {
            #[serde(rename = "req")]
            Request { id: String, method: String },
            #[serde(rename = "res")]
            Response { id: String, result: () },
        }

        assert_eq!(
            to_string(&Message::Request {
                id: "0".to_owned(),
                method: "post".to_owned()
            }),
            Ok("{\n  req: {\n    id: \"0\",\n    method: \"post\",\n  },\n}".to_owned())
        );
        assert_eq!(
            to_string(&Message::Response {
                id: "0".to_owned(),
                result: (),
            }),
            Ok("{\n  res: {\n    id: \"0\",\n    result: null,\n  },\n}".to_owned())
        );
    }

    // https://serde.rs/enum-representations.html#internally-tagged
    {
        #[derive(Serialize)]
        #[serde(tag = "type")]
        enum Message {
            #[serde(rename = "req")]
            Request { id: String, method: String },
            #[serde(rename = "res")]
            Response { id: String, result: () },
        }

        assert_eq!(
            to_string(&Message::Request {
                id: "0".to_owned(),
                method: "post".to_owned()
            }),
            Ok("{\n  type: \"req\",\n  id: \"0\",\n  method: \"post\",\n}".to_owned())
        );
        assert_eq!(
            to_string(&Message::Response {
                id: "0".to_owned(),
                result: (),
            }),
            Ok("{\n  type: \"res\",\n  id: \"0\",\n  result: null,\n}".to_owned())
        );
    }

    // https://serde.rs/enum-representations.html#adjacently-tagged
    {
        #[derive(Serialize)]
        #[serde(tag = "type", content = "value")]
        enum Message {
            #[serde(rename = "req")]
            Request { id: String, method: String },
            #[serde(rename = "res")]
            Response { id: String, result: () },
        }

        assert_eq!(
            to_string(&Message::Request {
                id: "0".to_owned(),
                method: "post".to_owned()
            }),
            Ok(
                "{\n  type: \"req\",\n  value: {\n    id: \"0\",\n    method: \"post\",\n  },\n}"
                    .to_owned()
            )
        );
        assert_eq!(
            to_string(&Message::Response {
                id: "0".to_owned(),
                result: (),
            }),
            Ok(
                "{\n  type: \"res\",\n  value: {\n    id: \"0\",\n    result: null,\n  },\n}"
                    .to_owned()
            )
        );
    }

    // https://serde.rs/enum-representations.html#untagged
    {
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Message {
            #[serde(rename = "req")]
            Request { id: String, method: String },
            #[serde(rename = "res")]
            Response { id: String, result: () },
        }

        assert_eq!(
            to_string(&Message::Request {
                id: "0".to_owned(),
                method: "post".to_owned()
            }),
            Ok("{\n  id: \"0\",\n  method: \"post\",\n}".to_owned())
        );
        assert_eq!(
            to_string(&Message::Response {
                id: "0".to_owned(),
                result: (),
            }),
            Ok("{\n  id: \"0\",\n  result: null,\n}".to_owned())
        );
    }
}
