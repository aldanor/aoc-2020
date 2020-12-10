use arrayvec::ArrayVec;

use crate::utils::*;

#[inline]
fn parse_and_sort(mut s: &[u8]) -> ArrayVec<[u8; 256]> {
    let mut a = ArrayVec::new();
    while s.len() > 1 {
        unsafe { a.push_unchecked(parse_int_fast(&mut s, 1, 3)) };
    }
    quickersort::sort(&mut a);
    a
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u16 {
    let (_, n1, n3) = parse_and_sort(s)
        .iter()
        .fold((0, 0u8, 0u8), |(prev, n1, n3), &x| {
            let dx = x - prev;
            let n1 = n1.wrapping_add((dx == 1) as u8);
            let n3 = n3.wrapping_add((dx == 3) as u8);
            (x, n1, n3)
        });
    (n1 as u16) * ((n3 + 1) as u16)
}

#[inline]
pub fn part2(s: &[u8]) -> usize {
    let a = parse_and_sort(s);
    let mut m = [0usize; 256];
    for i in 0..4 {
        m[i] = 1;
    }
    for &x in &a {
        let n = m.get_at(x as _);
        for i in 1..=3 {
            m.add_at((x as usize) + i, n);
        }
    }
    m.get_at((a.get_last() + 3) as _)
}

#[test]
fn test_day10_part1() {
    assert_eq!(part1(input()), 2470);
}

#[test]
fn test_day10_part2() {
    assert_eq!(part2(input()), 1973822685184);
}
