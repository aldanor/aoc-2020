use criterion::{black_box, criterion_group, criterion_main, Criterion};

use aoc2020::*;

macro_rules! bench {
    ($c:expr, $path:path) => {{
        use $path::*;
        let s = input();
        $c.bench_function(concat!(stringify!($path), "::part01"), |b| {
            b.iter(|| black_box(part1(black_box(&s))))
        });
        $c.bench_function(concat!(stringify!($path), "::part02"), |b| {
            b.iter(|| black_box(part2(black_box(&s))))
        });
        $c.bench_function(concat!(stringify!($path), "::input"), |b| {
            b.iter(|| input().len())
        });
    }};
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench!(c, day01);
    bench!(c, day02);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
