use serde_derive::Deserialize;
use std::fs::File;
use std::io::prelude::*;

mod common;

use crate::common::deserializes_to;

#[test]
fn serializes_example_infinite() {
    // use serde_json::Value;
    let mut contents = String::new();
    File::open("tests/assets/infinite.json5")
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    #[derive(Deserialize, PartialEq, Debug)]
    struct InfiniteF64 {
        inf: f64,
        neg_inf: f64,
    }

    let expected_f64 = InfiniteF64 {
        inf: f64::INFINITY,
        neg_inf: f64::NEG_INFINITY,
    };

    deserializes_to(&contents, expected_f64);

    #[derive(Deserialize, PartialEq, Debug)]
    struct InfiniteF32 {
        inf: f32,
        neg_inf: f32,
    }
    let expected_f32 = InfiniteF32 {
        inf: f32::INFINITY,
        neg_inf: f32::NEG_INFINITY,
    };

    deserializes_to(&contents, expected_f32)
}

#[test]
fn serializes_example_nan() {
    // use serde_json::Value;
    let mut contents = String::new();
    File::open("tests/assets/nan.json5")
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    #[derive(Deserialize, PartialEq, Debug)]
    struct NanF64 {
        nan: f64,
        neg_nan: f64,
    }

    match json5::from_str::<NanF64>(&contents) {
        Ok(value) => assert!(value.nan.is_nan() && value.neg_nan.is_nan()),
        Err(err) => panic!(format!("{}", err)),
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct NanF32 {
        nan: f32,
        neg_nan: f32,
    }

    match json5::from_str::<NanF32>(&contents) {
        Ok(value) => assert!(value.nan.is_nan() && value.neg_nan.is_nan()),
        Err(err) => panic!(format!("{}", err)),
    }
}
