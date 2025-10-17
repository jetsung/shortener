use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::Rng;
use std::hint::black_box;

/// Benchmark for short code generation
fn generate_code(length: usize, charset: &str) -> String {
    let mut rng = rand::rng();
    let chars: Vec<char> = charset.chars().collect();
    let charset_len = chars.len();

    (0..length)
        .map(|_| chars[rng.random_range(0..charset_len)])
        .collect()
}

fn benchmark_code_generation(c: &mut Criterion) {
    let charset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let mut group = c.benchmark_group("code_generation");

    for length in [4, 6, 8, 10, 12, 16].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(length), length, |b, &length| {
            b.iter(|| generate_code(black_box(length), black_box(charset)));
        });
    }

    group.finish();
}

fn benchmark_code_generation_different_charsets(c: &mut Criterion) {
    let charsets = [
        ("numeric", "0123456789"),
        ("lowercase", "abcdefghijklmnopqrstuvwxyz"),
        (
            "alphanumeric",
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        ),
        (
            "extended",
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_",
        ),
    ];

    let mut group = c.benchmark_group("code_generation_charsets");

    for (name, charset) in charsets.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), charset, |b, charset| {
            b.iter(|| generate_code(black_box(6), black_box(charset)));
        });
    }

    group.finish();
}

fn benchmark_code_uniqueness_check(c: &mut Criterion) {
    use std::collections::HashSet;

    c.bench_function("code_uniqueness_1000", |b| {
        b.iter(|| {
            let charset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
            let mut codes = HashSet::new();

            for _ in 0..1000 {
                let code = generate_code(6, charset);
                codes.insert(code);
            }

            black_box(codes)
        });
    });
}

criterion_group!(
    benches,
    benchmark_code_generation,
    benchmark_code_generation_different_charsets,
    benchmark_code_uniqueness_check
);
criterion_main!(benches);
