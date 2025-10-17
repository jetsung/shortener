use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// Simple URL validation (same as in service)
fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Code validation
fn is_valid_code(code: &str, charset: &str) -> bool {
    if code.is_empty() || code.len() > 16 {
        return false;
    }

    let charset_set: std::collections::HashSet<char> = charset.chars().collect();
    code.chars().all(|c| charset_set.contains(&c))
}

fn benchmark_url_validation(c: &mut Criterion) {
    let valid_urls = vec![
        "http://example.com",
        "https://example.com",
        "https://example.com/path",
        "https://example.com/path?query=value",
        "https://subdomain.example.com",
    ];

    let invalid_urls = vec![
        "ftp://example.com",
        "example.com",
        "www.example.com",
        "",
        "javascript:alert(1)",
    ];

    c.bench_function("validate_valid_urls", |b| {
        b.iter(|| {
            for url in &valid_urls {
                black_box(is_valid_url(url));
            }
        });
    });

    c.bench_function("validate_invalid_urls", |b| {
        b.iter(|| {
            for url in &invalid_urls {
                black_box(is_valid_url(url));
            }
        });
    });
}

fn benchmark_code_validation(c: &mut Criterion) {
    let charset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let valid_codes = vec!["abc123", "test", "ABC", "123456", "aB1cD2"];

    let invalid_codes = vec![
        "",
        "abc-123",
        "abc@123",
        "12345678901234567", // Too long
        "abc!def",
    ];

    c.bench_function("validate_valid_codes", |b| {
        b.iter(|| {
            for code in &valid_codes {
                black_box(is_valid_code(code, charset));
            }
        });
    });

    c.bench_function("validate_invalid_codes", |b| {
        b.iter(|| {
            for code in &invalid_codes {
                black_box(is_valid_code(code, charset));
            }
        });
    });
}

fn benchmark_charset_creation(c: &mut Criterion) {
    let charset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    c.bench_function("create_charset_hashset", |b| {
        b.iter(|| {
            let charset_set: std::collections::HashSet<char> = black_box(charset).chars().collect();
            black_box(charset_set)
        });
    });
}

criterion_group!(
    benches,
    benchmark_url_validation,
    benchmark_code_validation,
    benchmark_charset_creation
);
criterion_main!(benches);
