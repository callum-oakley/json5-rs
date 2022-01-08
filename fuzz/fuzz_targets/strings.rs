#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    if let Ok(value) = json5::from_str::<serde_json::Value>(data) {
        json5::to_string(&value).expect("serialization failed");
    }
});
