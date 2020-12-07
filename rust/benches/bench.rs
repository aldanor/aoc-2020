use criterion::{black_box, criterion_group, criterion_main, Criterion};

use aoc2020::*;

macro_rules! bench {
    ($c:expr, $path:path) => {{
        use $path::*;
        let s = input();
        $c.bench_function(concat!(stringify!($path), "::part1"), |b| {
            b.iter(|| black_box(part1(black_box(&s))))
        });
        $c.bench_function(concat!(stringify!($path), "::part2"), |b| {
            b.iter(|| black_box(part2(black_box(&s))))
        });
    }};
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench!(c, day01);
    bench!(c, day02);
    bench!(c, day03);
    bench!(c, day04);
    bench!(c, day05);
    bench!(c, day06);
    bench!(c, day07);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
