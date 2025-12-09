use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput, criterion_group,
    criterion_main,
};
use serde_json::Value;

fn criterion_benchmark(c: &mut Criterion) {
    let data = ["small.json", "medium.json", "large.json"].map(|file| {
        (
            file,
            std::fs::read_to_string(format!("benches/data/{file}")).unwrap(),
        )
    });

    let mut group = c.benchmark_group("from_str");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for (file, input) in data {
        group.sample_size(10);
        group.throughput(Throughput::Bytes(u64::try_from(input.len()).unwrap()));
        group.bench_with_input(
            BenchmarkId::new("callum-oakley/json5-rs", file),
            &input,
            |b, input| {
                b.iter(|| json5::from_str::<Value>(input).unwrap());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("spyoungtech/json-five-rs", file),
            &input,
            |b, input| {
                b.iter(|| json_five::from_str::<Value>(input).unwrap());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("google/serde_json5", file),
            &input,
            |b, input| {
                b.iter(|| serde_json5::from_str::<Value>(input).unwrap());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("serde-rs/json", file),
            &input,
            |b, input| {
                b.iter(|| serde_json::from_str::<Value>(input).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
