extern crate json5;
extern crate serde;

use std::f64;

#[allow(dead_code)]
pub fn parses_to<'a, T>(s: &'a str, v: T)
where
    T: ::std::fmt::Debug + ::std::cmp::PartialEq + serde::de::Deserialize<'a>,
{
    match json5::from_str::<T>(s) {
        Ok(value) => assert_eq!(value, v),
        Err(err) => panic!(format!("{}", err)),
    }
}

#[allow(dead_code)]
pub fn parses_to_nan<'a>(s: &'a str) {
    match json5::from_str::<f64>(s) {
        Ok(value) => assert!(value.is_nan()),
        Err(err) => panic!(format!("{}", err)),
    }
}
