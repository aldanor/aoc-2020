use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> i32 {
    let t0 = parse_int_fast::<i32>(&mut s, 1, 8);
    let (mut t_best, mut n_best) = (i32::MAX, 0);
    while s.len() > 1 {
        if s.get_first() == b'x' {
            s = s.advance(2);
            continue;
        }
        let n = parse_int_fast::<i32>(&mut s, 1, 3);
        let t = ((t0 + n - 1) / n) * n;
        if t < t_best {
            t_best = t;
            n_best = n;
        }
    }
    n_best * (t_best - t0)
}

#[inline]
fn bezout(a: i64, b: i64) -> (i64, i64) {
    // compute Bezout coefficients using extended Euclidean algorithm
    struct X(i64, i64, i64);
    let mut x0 = X(a, 1, 0);
    let mut x1 = X(b, 0, 1);
    while x1.0 != 0 {
        let q = x0.0 / x1.0;
        let x = X(x0.0 - q * x1.0, x0.1 - q * x1.1, x0.2 - q * x1.2);
        x0 = x1;
        x1 = x;
    }
    (x0.1, x0.2)
}

#[inline]
fn solve_crt_pair((a1, n1): (i64, i64), (a2, n2): (i64, i64)) -> (i64, i64) {
    // solve C.R.T. for a pair: x = a1 mod n1 and x = a2 mod n2
    let (m1, m2) = bezout(n1, n2);
    let n = n1 * n2;
    // use i128's to avoid temporary overflows
    let a1m2n2 = (a1 as i128) * (m2 as i128) * (n2 as i128);
    let a2m1n1 = (a2 as i128) * (m1 as i128) * (n1 as i128);
    let a = ((a1m2n2 + a2m1n1) % (n as i128)) as i64;
    (a, n)
}

#[inline]
pub fn part2(mut s: &[u8]) -> i64 {
    s = s.advance(1 + memchr::memchr(b'\n', s).unwrap());
    let mut pair = (0, parse_int_fast::<i64>(&mut s, 1, 3));
    let mut i = 0i64;
    while s.len() > 1 {
        i += 1;
        if s.get_first() == b'x' {
            s = s.advance(2);
            continue;
        }
        let n = parse_int_fast::<i64>(&mut s, 1, 3);
        let a = ((n - i) % n + n) % n; // double % to ensure it's always non-negative
        pair = solve_crt_pair(pair, (a, n));
    }
    pair.0
}

#[test]
fn test_day13_part1() {
    assert_eq!(part1(input()), 6568);
}

#[test]
fn test_day13_part2() {
    assert_eq!(part2(input()), 554865447501099);
}
