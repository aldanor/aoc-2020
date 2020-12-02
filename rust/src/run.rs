use aoc2020::day01;

macro_rules! run_day {
    ($day:path) => {{
        use $day::*;
        println!("{}: part1 = {}, part2 = {}", stringify!($day), part1(&input()), part2(&input()));
    }};
}

fn main() {
    run_day!(day01);
}
