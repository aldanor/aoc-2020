use std::hint::unreachable_unchecked;
use std::slice::from_raw_parts;

use memchr::memchr;

#[derive(Clone, Debug, Default)]
pub struct Password<'a> {
    pub pass: &'a [u8],
    pub n1: u8,
    pub n2: u8,
    pub char: u8,
}

#[inline]
unsafe fn parse_u8_superfast(s: &mut *const u8) -> u8 {
    let d0 = (**s).wrapping_sub(b'0');
    *s = s.add(1);
    let d1 = (**s).wrapping_sub(b'0');
    if d1 < 10 {
        *s = s.add(1);
        d0 * 10 + d1
    } else {
        d0
    }
}

impl<'a> Password<'a> {
    #[inline]
    pub unsafe fn parse_superfast(s: &mut &'a [u8]) -> Self {
        let mut p = s.as_ptr();
        let n1 = parse_u8_superfast(&mut p);
        p = p.add(1);
        let n2 = parse_u8_superfast(&mut p);
        p = p.add(1);
        let char = *p;
        p = p.add(3);
        let i = memchr(b'\n', from_raw_parts(p, 32)).unwrap_or_else(|| unreachable_unchecked());
        let pass = from_raw_parts(p, i);
        p = p.add(i + 1);
        *s = from_raw_parts(p, s.len() - (p.offset_from(s.as_ptr()) as usize));
        Self { pass, n1, n2, char }
    }
}

pub fn input() -> &'static [u8] {
    static INPUT: &[u8] = include_bytes!("input.txt");
    INPUT
}

#[inline]
pub fn part1(mut s: &[u8]) -> u16 {
    let mut valid = 0u16;
    while s.len() > 1 {
        let p = unsafe { Password::parse_superfast(&mut s) };
        let n = p
            .pass
            .into_iter()
            .take(32)
            .map(|&c| (c == p.char) as u8)
            .sum::<u8>();
        valid = valid.wrapping_add((n >= p.n1 && n <= p.n2) as u16);
    }
    valid
}

#[inline]
pub fn part2(mut s: &[u8]) -> u16 {
    let mut valid = 0;
    while s.len() > 1 {
        let p = unsafe { Password::parse_superfast(&mut s) };
        let match1 = unsafe { *p.pass.get_unchecked((p.n1 - 1) as usize) == p.char };
        let match2 = unsafe { *p.pass.get_unchecked((p.n2 - 1) as usize) == p.char };
        valid += (match1 != match2) as u16;
    }
    valid
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 477);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 686);
}
