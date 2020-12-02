use std::convert::TryFrom;
use std::str::FromStr;

use arrayvec::ArrayVec;

#[derive(Clone, Debug)]
pub struct Password {
    pub n1: u8,
    pub n2: u8,
    pub char: u8,
    pub pass: ArrayVec<[u8; 24]>,
}

impl FromStr for Password {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok((|| {
            let mut it = s.split(": ");
            let rule = it.next()?;
            let pass = it.next()?;
            let mut it = rule.split(' ');
            let minmax = it.next()?;
            let char = it.next()?;
            let mut it = minmax.split('-');
            let min = it.next()?;
            let max = it.next()?;
            Some(Self {
                n1: min.parse().unwrap(),
                n2: max.parse().unwrap(),
                char: char.as_bytes()[0],
                pass: ArrayVec::try_from(pass.as_bytes()).unwrap(),
            })
        })()
        .unwrap())
    }
}

pub fn input() -> Vec<Password> {
    include_str!("input.txt")
        .trim()
        .lines()
        .map(|l| l.parse().unwrap())
        .collect()
}

pub fn part1(s: &[Password]) -> u16 {
    let mut valid = 0u16;
    for p in s {
        let mut n = 0u8;
        for &c in &p.pass {
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
