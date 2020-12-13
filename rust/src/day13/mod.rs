use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> i32 {
    let t0 = parse_int_fast::<i32>(&mut s, 1, 8);
    let mut n_best = parse_int_fast::<i32>(&mut s, 1, 3);
    let mut m_best = t0 % n_best;
    while s.len() != 0 {
        while s.get_first() == b'x' {
            s = s.advance(2);
        }
        let n = parse_int_fast::<i32>(&mut s, 1, 3);
        let m = t0 % n;
        if m > m_best {
            m_best = m;
            n_best = n;
        }
    }
    n_best * (n_best - m_best)
}

const MAX_DIV: usize = 1024;

#[ctor::ctor]
static DIVS: [[i16; MAX_DIV]; MAX_DIV] = {
    let mut divs = [[0; MAX_DIV]; MAX_DIV];
    for i in 0..MAX_DIV {
        for j in 0..MAX_DIV {
            divs[i][j] = (i as i16) / (j as i16).max(1);
        }
    }
    divs
};

#[inline]
fn bezout(a: i64, b: i16) -> i16 {
    // compute Bezout coefficients using extended Euclidean algorithm
    // note: a will be big (and first q/r) but the rest of the numbers are very small
    // note (!): we will only return the first bezout coefficient, don't need the second one
    struct X(i16, i16);
    // do the first step manually, to get rid of big quotients
    let q = a / (b as i64); // only the first q is big
    let r = (a - q * (b as i64)) as i16; // the first r is already small
    if r == 0 {
        return 0;
    }
    let mut x0 = X(b, 0);
    let mut x1 = X(r, 1);
    // now do the 2nd step onwards, where quotient/remainder will be small
    loop {
        let q = DIVS[x0.0 as usize][x1.0 as usize]; // this q is small
        let r = x0.0 - q * x1.0; // this r is also small
        if r == 0 {
            break x1.1;
        }
        let x = X(r, x0.1 - q * x1.1);
        x0 = x1;
        x1 = x;
    }
}

#[inline]
fn solve_crt_pair((a1, n1): (i64, i64), (a2, n2): (i16, i16)) -> (i64, i64) {
    // solve C.R.T. for a pair: x = a1 mod n1 and x = a2 mod n2
    let m1 = bezout(n1, n2);
    // because (1): (a1 * m2 * n2 + a2 * m1 + n1) = a1 + (a2 - a1) * m1 * n1
    // because (2): (x * n1) % (n1 * n2) = n1 * (x % n2)
    let n = n1 * (n2 as i64);
    let a = a1 % n + n1 * ((((a2 as i64) - a1) * (m1 as i64)) % (n2 as i64));
    (a, n)
}

#[inline]
pub fn part2(mut s: &[u8]) -> i64 {
    s = s.advance(1 + memchr::memchr(b'\n', s).unwrap());
    let mut pair = (0, parse_int_fast::<i16>(&mut s, 1, 3) as i64);
    let mut i = 0i16;
    while s.len() > 1 {
        i += 1;
        if s.get_first() == b'x' {
            s = s.advance(2);
            continue;
        }
        let n = parse_int_fast::<i16>(&mut s, 1, 3);
        pair = solve_crt_pair(pair, (n - i, n));
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
