use iai::black_box;

use benchmarks::benchmarks::*;

fn i32_cell_new() -> I32Cell {
    benchmarks::benchmarks::i32_cell_new(black_box(66))
}

fn i32_cell_try_new_ok() -> Result<I32Cell, Box<u32>> {
    benchmarks::benchmarks::i32_cell_try_new_ok(black_box(66))
}

fn i32_list_1k() -> i32 {
    i32_list(black_box(1_000))
}

fn i32_list_100k() -> i32 {
    i32_list(black_box(100_000))
}

fn i32_list_1m() -> i32 {
    i32_list(black_box(1_000_000))
}

fn i32_random_1k() -> i32 {
    i32_random(black_box(1_000))
}

fn i32_random_100k() -> i32 {
    i32_random(black_box(100_000))
}

fn i32_random_1m() -> i32 {
    i32_random(black_box(1_000_000))
}

fn string_cell_new() -> StringCell {
    benchmarks::benchmarks::string_cell_new(black_box("cantelope for mars yes no".into()))
}

fn string_cell_try_new_ok() -> Result<StringCell, Box<u32>> {
    benchmarks::benchmarks::string_cell_try_new_ok(black_box("short".into()))
}

fn string_list_1k() -> i32 {
    string_list(black_box(1_000))
}

fn string_list_100k() -> i32 {
    string_list(black_box(100_000))
}

fn string_random_1k() -> i32 {
    string_random(black_box(1_000))
}

fn string_random_100k() -> i32 {
    string_random(black_box(100_000))
}

iai::main!(
    i32_cell_new,
    i32_cell_try_new_ok,
    i32_list_1k,
    i32_list_100k,
    i32_list_1m,
    i32_random_1k,
    i32_random_100k,
    i32_random_1m,
    string_cell_new,
    string_cell_try_new_ok,
    string_list_1k,
    string_list_100k,
    string_random_1k,
    string_random_100k,
);
