use std::iter;

use rustc_hash::FxHashMap;

use crate::utils::*;

type N = u64;

#[inline]
fn modmul(a: N, b: N, n: N) -> N {
    ((a as u64 * b as u64) % (n as u64)) as N
}

#[inline]
fn modpow(mut base: N, mut exp: N, n: N) -> N {
    let mut res = 1;
    base %= n;
    while exp != 0 {
        if exp & 1 != 0 {
            res = modmul(res, base, n);
        }
        exp >>= 1;
        base = modmul(base, base, n);
    }
    res
}

#[inline]
fn inverse_euclidean(a: N, n: N) -> Option<N> {
    // modular inverse: find x such that (a * x) % n = 1
    let (a, n) = (a as i32, n as i32);
    let mut t = (0, 1);
    let mut r = (n, a);
    while r.1 != 0 {
        let q = r.0 / r.1;
        t = (t.1, t.0 - q * t.1);
        r = (r.1, r.0 - q * r.1);
    }
    if r.0 > 1 {
        None
    } else {
        Some((t.0 + n) as _)
    }
}

#[inline]
fn iter_modmul(init: N, base: N, n: N) -> impl Iterator<Item = N> {
    let mut e = init;
    iter::from_fn(move || {
        let out = e;
        e = modmul(e, base, n);
        Some(out)
    })
}

#[inline]
fn babystep_giantstep(g: N, h: N, n: N) -> Option<N> {
    // solve (g ^ x) % n = h using baby-step-giant-step algorithm
    let mut table = FxHashMap::with_capacity_and_hasher(1 << 13, Default::default());
    let m = f64::sqrt(n as _).ceil() as usize;
    let mut z = 1;
    for (i, e) in iter_modmul(1, g, n).take(m).enumerate() {
        table.insert(e, i as N);
        z = e;
    }
    z = modmul(z, g, n);
    let f = inverse_euclidean(z, n)?; // could also modpow(g, n - m - 1, n), but slower
    iter_modmul(h, f, n)
        .take(m)
        .enumerate()
        .filter_map(|(i, e)| table.get(&e).map(|x| (i * m) as N + x))
        .next()
}

fn parse_input(mut s: &[u8]) -> (N, N) {
    (parse_int_fast(&mut s, 1, 8), parse_int_fast(&mut s, 1, 8))
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> N {
    const M: N = 20201227;
    let (public_key_1, public_key_2) = parse_input(s);
    let loop_size_1 = babystep_giantstep(7, public_key_1, M).unwrap();
    if cfg!(debug_assertions) {
        let loop_size_2 = babystep_giantstep(7, public_key_2, M).unwrap();
        let encryption_key_1 = modpow(public_key_2, loop_size_1, M);
        let encryption_key_2 = modpow(public_key_1, loop_size_2, M);
        assert_eq!(encryption_key_1, encryption_key_2);
    }
    modpow(public_key_2, loop_size_1, M)
}

#[inline]
pub fn part2(_: &[u8]) -> usize {
    0
}

#[test]
fn test_day25_part1() {
    assert_eq!(part1(input()), 5025281);
}

#[test]
fn test_day25_part2() {
    assert_eq!(part2(input()), 0);
}
