use criterion::{black_box, criterion_group, criterion_main, Criterion};

use aoc2020::day01;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("day01_1", |b| {
        let s = day01::input();
        b.iter(|| black_box(day01::part1(black_box(&s))))
    });
    c.bench_function("day01_2", |b| {
        let s = day01::input();
        b.iter(|| black_box(day01::part2(black_box(&s))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
