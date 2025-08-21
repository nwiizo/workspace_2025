//! Performance benchmarks for process operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use linux_process_rs::process::ProcessBuilder;
use std::time::Duration;

fn benchmark_process_spawn(c: &mut Criterion) {
    c.bench_function("process_spawn", |b| {
        b.iter(|| {
            let result = ProcessBuilder::new("echo").arg("test").output();
            let _ = black_box(result);
        });
    });
}

fn benchmark_process_with_timeout(c: &mut Criterion) {
    c.bench_function("process_with_timeout", |b| {
        b.iter(|| {
            let result = ProcessBuilder::new("echo")
                .arg("test")
                .timeout(Duration::from_secs(1))
                .output();
            let _ = black_box(result);
        });
    });
}

fn benchmark_input_validation(c: &mut Criterion) {
    use linux_process_rs::process::validate_input;

    c.bench_function("input_validation_safe", |b| {
        b.iter(|| {
            let result = validate_input(black_box("normal_file.txt"));
            let _ = black_box(result);
        });
    });

    c.bench_function("input_validation_unsafe", |b| {
        b.iter(|| {
            let result = validate_input(black_box("file.txt; rm -rf /"));
            let _ = black_box(result);
        });
    });
}

criterion_group!(
    benches,
    benchmark_process_spawn,
    benchmark_process_with_timeout,
    benchmark_input_validation
);
criterion_main!(benches);
