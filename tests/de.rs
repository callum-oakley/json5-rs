extern crate json5;
#[macro_use]
extern crate serde_derive;

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
fn parse_bytes() {
    // TODO
}

#[test]
#[ignore] // currently unsupported
fn parse_byte_buf() {
    // TODO
}

#[test]
fn parse_option() {
    parses_to::<Option<i32>>("null", None);
    parses_to("42", Some(42));
    parses_to("42", Some(Some(42)));
}

#[test]
fn parse_unit() {
    parses_to("null", ());
}

#[test]
fn parse_unit_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct A;
    parses_to("null", A);
}

#[test]
fn parse_newtype_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct A(i32);

    #[derive(Deserialize, PartialEq, Debug)]
    struct B(f64);

    parses_to("42", A(42));
    parses_to("42", B(42.));
}
