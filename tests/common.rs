use json5::{Error, Location};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use std::f64;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum Val {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Val>),
    Object(HashMap<String, Val>),
}

#[allow(unused)]
#[track_caller]
pub fn deserializes_to<'a, T>(s: &'a str, v: T)
where
    T: ::std::fmt::Debug + ::std::cmp::PartialEq + serde::de::Deserialize<'a>,
{
    assert_eq!(json5::from_str::<T>(s).expect("deserialization failed"), v);
}

#[allow(unused)]
#[track_caller]
pub fn deserializes_to_nan_f32(s: &str) {
    let float = json5::from_str::<f32>(s).expect("f32 deserialization failed");
    if !float.is_nan() {
        panic!("assertion failed: {}.is_nan()", float);
    }
}

#[allow(unused)]
#[track_caller]
pub fn deserializes_to_nan_f64(s: &str) {
    let float = json5::from_str::<f64>(s).expect("f64 deserialization failed");
    if !float.is_nan() {
        panic!("assertion failed: {}.is_nan()", float);
    }
}

#[allow(unused)]
#[track_caller]
pub fn deserializes_with_error<'a, T>(s: &'a str, error_expected: Error)
where
    T: ::std::fmt::Debug + ::std::cmp::PartialEq + serde::de::Deserialize<'a>,
{
    assert_eq!(
        json5::from_str::<T>(s).expect_err("deserialization succeeded"),
        error_expected
    );
}

#[allow(unused)]
#[track_caller]
pub fn serializes_to<T>(v: T, s: &'static str)
where
    T: ::std::fmt::Debug + ::std::cmp::PartialEq + serde::ser::Serialize,
{
    assert_eq!(json5::to_string::<T>(&v).expect("serialization failed"), s);
}

#[allow(unused)]
pub fn make_error(msg: impl Into<String>, line: usize, column: usize) -> Error {
    Error::Message {
        msg: msg.into(),
        location: Some(Location { line, column }),
    }
}
