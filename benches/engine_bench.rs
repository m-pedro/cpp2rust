use cpp2rust::aggregator::aggregate_buffer_par;
use cpp2rust::parser::parse_log_line_rs;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_single_line_parser(c: &mut Criterion) {
    let sample_line =
        b"10.0.0.42 - - [25/Oct/2026:12:34:56 +0000] \"POST /api/v1/checkout HTTP/1.1\" 200 2048 14.75";

    let mut group = c.benchmark_group("parser");
    group.throughput(Throughput::Bytes(sample_line.len() as u64));
    group.bench_function("parse_log_line_rs", |b| {
        b.iter(|| {
            let rec = parse_log_line_rs(black_box(sample_line)).unwrap();
            black_box(rec);
        })
    });
    group.finish();
}

fn bench_parallel_aggregation(c: &mut Criterion) {
    let single_line =
        "192.168.1.10 - - [25/Oct/2026:00:00:01 +0000] \"GET /api/v1/users HTTP/1.1\" 200 1024 8.50\n";
    let synthetic_10k = single_line.repeat(10_000).into_bytes();

    let mut group = c.benchmark_group("aggregator");
    group.throughput(Throughput::Bytes(synthetic_10k.len() as u64));
    group.bench_function("aggregate_buffer_par_10k_lines", |b| {
        b.iter(|| {
            let stats = aggregate_buffer_par(black_box(&synthetic_10k));
            black_box(stats);
        })
    });
    group.finish();
}

criterion_group!(benches, bench_single_line_parser, bench_parallel_aggregation);
criterion_main!(benches);
