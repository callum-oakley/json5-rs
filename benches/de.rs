use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput, criterion_group,
    criterion_main,
};
use serde_json::Value;

fn criterion_benchmark(c: &mut Criterion) {
    // JSON5 strings to deserialize
    let data = ["small.json", "medium.json", "large.json"].map(|file| {
        (
            file,
            json5::to_string(
                &json5::from_str::<Value>(
                    &std::fs::read_to_string(format!("benches/data/{file}")).unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });

    let mut group = c.benchmark_group("de");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for (file, input) in data {
        group.sample_size(10);
        group.throughput(Throughput::Bytes(u64::try_from(input.len()).unwrap()));
        group.bench_with_input(BenchmarkId::from_parameter(file), &input, |b, input| {
            b.iter(|| json5::from_str::<Value>(input).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
