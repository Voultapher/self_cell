use criterion::{black_box, criterion_group, criterion_main, Criterion};

use benchmarks::benchmarks::*;

pub fn i32_benchmarks(c: &mut Criterion) {
    c.bench_function("i32_cell_new", |b| b.iter(|| i32_cell_new(black_box(66))));
    c.bench_function("i32_cell_try_new_ok", |b| {
        b.iter(|| i32_cell_try_new_ok(black_box(66)))
    });
    c.bench_function("i32_list_1k", |b| b.iter(|| i32_list(black_box(1_000))));
    c.bench_function("i32_list_100k", |b| b.iter(|| i32_list(black_box(100_000))));
    c.bench_function("i32_list_1m", |b| b.iter(|| i32_list(black_box(1_000_000))));
    c.bench_function("i32_random_1k", |b| b.iter(|| i32_random(black_box(1_000))));
    c.bench_function("i32_random_100k", |b| {
        b.iter(|| i32_random(black_box(100_000)))
    });
    c.bench_function("i32_random_1m", |b| {
        b.iter(|| i32_random(black_box(1_000_000)))
    });
}

criterion_group!(i32_benches, i32_benchmarks);

pub fn string_benchmarks(c: &mut Criterion) {
    c.bench_function("string_cell_new", |b| {
        b.iter(|| string_cell_new(black_box("some longer string yes maybe yes".into())))
    });
    c.bench_function("string_cell_try_new_ok", |b| {
        b.iter(|| string_cell_try_new_ok(black_box("short".into())))
    });
    c.bench_function("string_list_1k", |b| {
        b.iter(|| string_list(black_box(1_000)))
    });
    c.bench_function("string_list_100k", |b| {
        b.iter(|| string_list(black_box(100_000)))
    });
    c.bench_function("string_random_1k", |b| {
        b.iter(|| string_random(black_box(1_000)))
    });
    c.bench_function("string_random_100k", |b| {
        b.iter(|| string_random(black_box(100_000)))
    });
}

criterion_group!(string_benches, string_benchmarks);

criterion_main!(i32_benches, string_benches);
