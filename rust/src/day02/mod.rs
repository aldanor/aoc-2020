use arrayvec::ArrayVec;

use crate::parse::{expect_char, expect_str, parse_char, parse_uint, parse_until_eol};

#[derive(Clone, Debug, Default)]
pub struct Password<'a> {
    pub pass: &'a [u8],
    pub n1: u8,
    pub n2: u8,
    pub char: u8,
}

impl<'a> Password<'a> {
    pub fn parse(s: &'a [u8]) -> Option<Self> {
        let (s, n1) = parse_uint(s)?;
        let (s, _) = expect_char(b'-')(s)?;
        let (s, n2) = parse_uint(s)?;
        let (s, _) = expect_char(b' ')(s)?;
        let (s, char) = parse_char(s)?;
        let (pass, _) = expect_str(b": ")(s)?;
        Some(Self { pass, n1, n2, char })
    }

    pub fn parse_all(s: &'a str) -> Option<ArrayVec<[Self; 1024]>> {
        let mut s = s.as_bytes();
        let mut vec = ArrayVec::<[Self; 1024]>::default();
        while !s.is_empty() {
            let res = parse_until_eol(s)?;
            if res.1.is_empty() {
                break;
            }
            unsafe { vec.push_unchecked(Self::parse(res.1)?) };
            s = res.0;
        }
        Some(vec)
    }
}

pub fn input() -> ArrayVec<[Password<'static>; 1024]> {
    static INPUT: &str = include_str!("input.txt");
    Password::parse_all(INPUT).unwrap()
}

pub fn part1(s: &[Password]) -> u16 {
    let mut valid = 0u16;
    for p in s {
        let mut n = 0u8;
        for &c in p.pass.iter() {
            n += (c == p.char) as u8;
        }
        valid += (n >= p.n1 && n <= p.n2) as u16;
    }
    valid
}

pub fn part2(s: &[Password]) -> u16 {
    s.iter()
        .map(|p| {
            let match1 = unsafe { *p.pass.get_unchecked((p.n1 - 1) as usize) == p.char };
            let match2 = unsafe { *p.pass.get_unchecked((p.n2 - 1) as usize) == p.char };
            (match1 != match2) as u16
        })
        .sum()
}
