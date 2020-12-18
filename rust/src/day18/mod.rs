use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

type N = u64;

#[inline(always)]
fn read_one(s: &mut &[u8]) -> u8 {
    let c = s.get_first();
    *s = s.advance(1);
    c
}

#[inline(always)]
fn next(s: &mut &[u8], f: impl Fn(&mut &[u8]) -> N) -> N {
    match read_one(s) {
        c if c == b'(' => f(s),
        c => (c - b'0') as N,
    }
}

#[inline(always)]
fn parse_expr_no_precedence(s: &mut &[u8]) -> N {
    let mut v = next(s, parse_expr_no_precedence);
    while !matches!(read_one(s), b')' | b'\n') {
        let op = s.get_first();
        *s = s.advance(2);
        let rhs = next(s, parse_expr_no_precedence);
        v = if op == b'+' { v + rhs } else { v * rhs };
    }
    v
}

#[inline(always)]
fn parse_expr_with_precedence(s: &mut &[u8]) -> N {
    let mut v = next(s, parse_expr_with_precedence);
    let mut accum = 1;
    while !matches!(read_one(s), b')' | b'\n') {
        let op = s.get_first();
        *s = s.advance(2);
        let rhs = next(s, parse_expr_with_precedence);
        v = if op == b'+' {
            v + rhs
        } else {
            accum *= v;
            rhs
        };
    }
    v * accum
}

#[inline]
pub fn part1(mut s: &[u8]) -> u64 {
    let mut total = 0;
    while !s.is_empty() {
        total += parse_expr_no_precedence(&mut s);
    }
    total
}

#[inline]
pub fn part2(mut s: &[u8]) -> u64 {
    let mut total = 0;
    while !s.is_empty() {
        total += parse_expr_with_precedence(&mut s);
    }
    total
}

#[test]
fn test_day18_part1() {
    assert_eq!(part1(input()), 2743012121210);
}

#[test]
fn test_day18_part2() {
    assert_eq!(part2(input()), 65658760783597);
}
