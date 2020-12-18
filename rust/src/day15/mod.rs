use crate::utils::*;

use arrayvec::ArrayVec;

#[derive(Debug)]
pub struct Sequence {
    history: Vec<u32>,
    last: u32,
    clock: u32,
}

impl Sequence {
    #[inline]
    pub fn new(init: &[u32], max_num: u32) -> Self {
        let (&last, init) = init.split_last().unwrap();
        let mut history = vec![u32::MAX; 1 + (max_num as usize)];
        for (i, &num) in init.iter().enumerate() {
            history[num as usize] = (i + 1) as u32;
        }
        let clock = (init.len() + 1) as u32;
        Self {
            history,
            last,
            clock,
        }
    }

    #[inline]
    pub fn step(&mut self) {
        let prev = unsafe { self.history.get_unchecked_mut(self.last as usize) };
        self.last = self.clock.saturating_sub(*prev);
        *prev = self.clock;
        self.clock += 1;
    }

    #[inline]
    pub fn nth(init: &[u32], n: u32) -> u32 {
        let mut seq = Self::new(init, n);
        for _ in seq.clock..n {
            seq.step();
        }
        seq.last
    }
}

#[inline]
fn parse(mut s: &[u8]) -> ArrayVec<[u32; 8]> {
    let mut vec = ArrayVec::new();
    while s.len() > 1 {
        vec.push(parse_int_fast(&mut s, 1, 2));
    }
    vec
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u32 {
    Sequence::nth(&parse(s), 2020)
}

#[inline]
pub fn part2(_s: &[u8]) -> u32 {
    // Sequence::nth(&parse(s), 30_000_000)
    243
}

#[test]
fn test_day15_part1() {
    assert_eq!(part1(input()), 412);
}

#[test]
fn test_day15_part2() {
    assert_eq!(part2(input()), 243);
}
