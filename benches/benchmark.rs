use criterion::Criterion;
use std::{
    fs,
    path::{Path, PathBuf},
};

const BENCHMARKS: [(&str, &str); 7] = [
    // Small benchmarks
    ("small_menu", "https://cdn.jsdelivr.net/gh/RichardHightower/json-parsers-benchmark@e6d09a817eafc50a5cad821e0743d565899639d9/data/menu.json"),
    ("small_rh", "https://cdn.jsdelivr.net/gh/RichardHightower/json-parsers-benchmark@e6d09a817eafc50a5cad821e0743d565899639d9/data/small.json"),

    // Medium benchmarks
    ("med_rh", "https://cdn.jsdelivr.net/gh/RichardHightower/json-parsers-benchmark@e6d09a817eafc50a5cad821e0743d565899639d9/data/medium.json"),
    ("med_webxml", "https://cdn.jsdelivr.net/gh/RichardHightower/json-parsers-benchmark@e6d09a817eafc50a5cad821e0743d565899639d9/data/webxml.json"),

    // Large benchmarks
    ("large_canada", "https://cdn.jsdelivr.net/gh/miloyip/nativejson-benchmark@c00ff27cd5b059b043c414b8a996938f8f9e9663/data/canada.json"),
    ("large_citm_catalog", "https://cdn.jsdelivr.net/gh/miloyip/nativejson-benchmark@c00ff27cd5b059b043c414b8a996938f8f9e9663/data/citm_catalog.json"),
    ("large_twitter", "https://cdn.jsdelivr.net/gh/miloyip/nativejson-benchmark@c00ff27cd5b059b043c414b8a996938f8f9e9663/data/twitter.json"),
];

fn main() {
    let mut c = Criterion::default().configure_from_args();

    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("benches")
            .join("data"),
    )
    .expect("could not go to benchmark data dir");

    let mut agent: Option<ureq::Agent> = None;

    // Read or download benchmark files
    let benches = BENCHMARKS.map(|(bench_name, url)| {
        let filename = PathBuf::from(format!("{}.json", bench_name));
        let error = match fs::read_to_string(&filename) {
            Ok(json) => return (bench_name, json),
            Err(e) => e,
        };
        eprintln!(
            "Failed to read benchmark file {}: {}",
            filename.display(),
            error
        );
        eprintln!("Falling back to downloading file from <{}>", url);

        let response = agent
            .get_or_insert_with(ureq::agent)
            .get(url)
            .call()
            .expect("could not download JSON file");

        if response.status() != 200 {
            panic!(
                "Failed to download JSON: response status was {}",
                response.status()
            );
        }

        let json = response
            .into_string()
            .expect("failed to read JSON response body");

        if let Err(e) = fs::write(&filename, &json) {
            eprintln!("warning: could not write JSON to file: {}", e);
        }

        (bench_name, json)
    });

    eprintln!("Successfully loaded all benchmark JSON files");

    for (bench_name, json) in benches {
        let value = json5::from_str::<serde_json::Value>(&json).expect("JSON benchmark data invalid");
        c.bench_function(&format!("deserialize_{}", bench_name), |b| {
            b.iter_with_large_drop(|| json5::from_str::<serde_json::Value>(&json).unwrap())
        });
        c.bench_function(&format!("serialize_{}", bench_name), |b| {
            b.iter(|| json5::to_string(&value).unwrap())
        });
    }
}
