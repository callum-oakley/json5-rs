extern crate json5;
#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

mod common;

use common::parses_to;

#[test]
fn parses_bool() {
    parses_to("true", true);
    parses_to("false", false);
}

#[test]
fn parses_i8() {
    let x: i8 = 42;
    parses_to("0x2A", x);
    parses_to("0x2a", x);
    parses_to("0X2A", x);
    parses_to("0X2a", x);
    parses_to("0x00002A", x);
    parses_to("42", x);
    parses_to("42.", x);
    parses_to("42.0", x);
    parses_to("42e0", x);
    parses_to("4.2e1", x);
    parses_to(".42e2", x);
    parses_to("0.42e2", x);
    parses_to("-42", -x);
    parses_to("-42.", -x);
    parses_to("-42.0", -x);
    parses_to("-42e0", -x);
    parses_to("-4.2e1", -x);
    parses_to("-.42e2", -x);
    parses_to("-0.42e2", -x);
}

#[test]
fn parses_u8() {
    let x: u8 = 42;
    parses_to("0x2A", x);
    parses_to("0x2a", x);
    parses_to("0X2A", x);
    parses_to("0X2a", x);
    parses_to("0x00002A", x);
    parses_to("42", x);
    parses_to("42.", x);
    parses_to("42.0", x);
    parses_to("42e0", x);
    parses_to("4.2e1", x);
    parses_to(".42e2", x);
    parses_to("0.42e2", x);
}

#[test]
fn parses_i16() {
    let x: i16 = 42;
    parses_to("0x2A", x);
    parses_to("0x2a", x);
    parses_to("0X2A", x);
    parses_to("0X2a", x);
    parses_to("0x00002A", x);
    parses_to("42", x);
    parses_to("42.", x);
    parses_to("42.0", x);
    parses_to("42e0", x);
    parses_to("4.2e1", x);
    parses_to(".42e2", x);
    parses_to("0.42e2", x);
    parses_to("-42", -x);
    parses_to("-42.", -x);
    parses_to("-42.0", -x);
    parses_to("-42e0", -x);
    parses_to("-4.2e1", -x);
    parses_to("-.42e2", -x);
    parses_to("-0.42e2", -x);
}

#[test]
fn parses_u16() {
    let x: u16 = 42;
    parses_to("0x2A", x);
    parses_to("0x2a", x);
    parses_to("0X2A", x);
    parses_to("0X2a", x);
    parses_to("0x00002A", x);
    parses_to("42", x);
    parses_to("42.", x);
    parses_to("42.0", x);
    parses_to("42e0", x);
    parses_to("4.2e1", x);
    parses_to(".42e2", x);
    parses_to("0.42e2", x);
}

#[test]
fn parses_i32() {
    let x: i32 = 42;
    parses_to("0x2A", x);
    parses_to("0x2a", x);
    parses_to("0X2A", x);
    parses_to("0X2a", x);
    parses_to("0x00002A", x);
    parses_to("42", x);
    parses_to("42.", x);
    parses_to("42.0", x);
    parses_to("42e0", x);
    parses_to("4.2e1", x);
    parses_to(".42e2", x);
    parses_to("0.42e2", x);
    parses_to("-42", -x);
    parses_to("-42.", -x);
    parses_to("-42.0", -x);
    parses_to("-42e0", -x);
    parses_to("-4.2e1", -x);
    parses_to("-.42e2", -x);
    parses_to("-0.42e2", -x);
}

#[test]
fn parses_u32() {
    let x: u32 = 42;
    parses_to("0x2A", x);
    parses_to("0x2a", x);
    parses_to("0X2A", x);
    parses_to("0X2a", x);
    parses_to("0x00002A", x);
    parses_to("42", x);
    parses_to("42.", x);
    parses_to("42.0", x);
    parses_to("42e0", x);
    parses_to("4.2e1", x);
    parses_to(".42e2", x);
    parses_to("0.42e2", x);
}

#[test]
fn parses_i64() {
    let x: i64 = 42;
    parses_to("0x2A", x);
    parses_to("0x2a", x);
    parses_to("0X2A", x);
    parses_to("0X2a", x);
    parses_to("0x00002A", x);
    parses_to("42", x);
    parses_to("42.", x);
    parses_to("42.0", x);
    parses_to("42e0", x);
    parses_to("4.2e1", x);
    parses_to(".42e2", x);
    parses_to("0.42e2", x);
    parses_to("-42", -x);
    parses_to("-42.", -x);
    parses_to("-42.0", -x);
    parses_to("-42e0", -x);
    parses_to("-4.2e1", -x);
    parses_to("-.42e2", -x);
    parses_to("-0.42e2", -x);
}

#[test]
fn parses_u64() {
    let x: u64 = 42;
    parses_to("0x2A", x);
    parses_to("0x2a", x);
    parses_to("0X2A", x);
    parses_to("0X2a", x);
    parses_to("0x00002A", x);
    parses_to("42", x);
    parses_to("42.", x);
    parses_to("42.0", x);
    parses_to("42e0", x);
    parses_to("4.2e1", x);
    parses_to(".42e2", x);
    parses_to("0.42e2", x);
}

#[test]
fn parses_f32() {
    let x: f32 = 42.42;
    parses_to("42.42", x);
    parses_to("42.42e0", x);
    parses_to("4.242e1", x);
    parses_to(".4242e2", x);
    parses_to("0.4242e2", x);
    parses_to("-42.42", -x);
    parses_to("-42.42", -x);
    parses_to("-42.42", -x);
    parses_to("-42.42e0", -x);
    parses_to("-4.242e1", -x);
    parses_to("-.4242e2", -x);
    parses_to("-0.4242e2", -x);
}

#[test]
fn parses_f64() {
    let x: f64 = 42.42;
    parses_to("42.42", x);
    parses_to("42.42e0", x);
    parses_to("4.242e1", x);
    parses_to(".4242e2", x);
    parses_to("0.4242e2", x);
    parses_to("-42.42", -x);
    parses_to("-42.42", -x);
    parses_to("-42.42", -x);
    parses_to("-42.42e0", -x);
    parses_to("-4.242e1", -x);
    parses_to("-.4242e2", -x);
    parses_to("-0.4242e2", -x);
}

#[test]
fn parses_char() {
    parses_to("'x'", 'x');
    parses_to("\"자\"", '자');
}

#[test]
#[ignore] // currently unsupported
fn parses_str() {
    parses_to("'Hello!'", "Hello!");
    parses_to("\"안녕하세요\"", "안녕하세요");
}

#[test]
fn parses_string() {
    parses_to("'Hello!'", "Hello!".to_owned());
    parses_to("\"안녕하세요\"", "안녕하세요".to_owned());
}

#[test]
#[ignore] // currently unsupported
fn parses_bytes() {
    // TODO
}

#[test]
#[ignore] // currently unsupported
fn parses_byte_buf() {
    // TODO
}

#[test]
fn parses_option() {
    parses_to::<Option<i32>>("null", None);
    parses_to("42", Some(42));
    parses_to("42", Some(Some(42)));
}

#[test]
fn parses_unit() {
    parses_to("null", ());
}

#[test]
fn parses_unit_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct A;
    parses_to("null", A);
}

#[test]
fn parses_newtype_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct A(i32);

    #[derive(Deserialize, PartialEq, Debug)]
    struct B(f64);

    parses_to("42", A(42));
    parses_to("42", B(42.));
}

#[test]
fn parses_seq() {
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum Val {
        Number(f64),
        Bool(bool),
        String(String),
    }

    parses_to("[1, 2, 3]", vec![1, 2, 3]);
    parses_to(
        "[42, true, 'hello']",
        vec![
            Val::Number(42.),
            Val::Bool(true),
            Val::String("hello".to_owned()),
        ],
    )
}

#[test]
fn parses_tuple() {
    parses_to("[1, 2, 3]", (1, 2, 3));
}

#[test]
fn parses_tuple_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct A(i32, f64);

    #[derive(Deserialize, PartialEq, Debug)]
    struct B(f64, i32);

    parses_to("[1, 2]", A(1, 2.));
    parses_to("[1, 2]", B(1., 2));
}

#[test]
fn parses_map() {
    let mut m = HashMap::new();
    m.insert("a".to_owned(), 1);
    m.insert("b".to_owned(), 2);
    m.insert("c".to_owned(), 3);

    parses_to("{ a: 1, 'b': 2, \"c\": 3 }", m);
}

#[test]
fn parses_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct S {
        a: i32,
        b: i32,
        c: i32,
    }

    parses_to("{ a: 1, 'b': 2, \"c\": 3 }", S { a: 1, b: 2, c: 3 });
}

#[test]
fn parses_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        A,
        B(i32),
        C(i32, i32),
        D { a: i32, b: i32 },
    }

    parses_to("'A'", E::A);
    parses_to("{ B: 2 }", E::B(2));
    parses_to("{ C: [3, 5] }", E::C(3, 5));
    parses_to("{ D: { a: 7, b: 11 } }", E::D { a: 7, b: 11 });
}

#[test]
fn parses_ignored() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct S {
        a: i32,
        b: i32,
    }

    parses_to("{ a: 1, ignored: 42, b: 2 }", S { a: 1, b: 2 });
}
