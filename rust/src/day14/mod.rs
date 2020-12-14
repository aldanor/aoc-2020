use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::iter;

use rustc_hash::FxHasher;

use crate::utils::*;

type Map = HashMap<u64, u64, BuildHasherDefault<FxHasher>>;

#[derive(Debug, Copy, Clone, Default)]
struct Mask {
    mask: u64,
    value: u64,
}

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Mask(Mask),
    Write { addr: u64, value: u64 },
}

#[inline]
fn parse_instructions<'a>(mut s: &'a [u8]) -> impl Iterator<Item = Instruction> + 'a {
    iter::from_fn(move || {
        if s.len() > 1 {
            if s.get_at(1) == b'e' {
                s = s.advance(4);
                let addr = parse_int_fast(&mut s, 1, 10);
                s = s.advance(3);
                let value = parse_int_fast(&mut s, 1, 10);
                Some(Instruction::Write { addr, value })
            } else {
                s = s.advance(7);
                let (mut mask, mut value) = (0, 0);
                for i in 0..36 {
                    let c = s.get_at(i).wrapping_sub(b'X');
                    let p = 35 - i;
                    mask |= ((c == 0) as u64) << p;
                    value |= ((c == b'1'.wrapping_sub(b'X')) as u64) << p;
                }
                s = s.advance(37);
                Some(Instruction::Mask(Mask { mask, value }))
            }
        } else {
            None
        }
    })
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u64 {
    let mut map = Map::with_capacity_and_hasher(1024, Default::default());
    let (mut masked, mut bitmask) = (0, 0);
    for instruction in parse_instructions(s) {
        match instruction {
            Instruction::Mask(mask) => {
                bitmask = mask.mask;
                masked = !mask.mask & mask.value;
            }
            Instruction::Write { addr, value } => {
                let value = (value & bitmask) | masked;
                *map.entry(addr).or_insert(value) = value;
            }
        }
    }
    map.values().sum()
}

#[inline]
pub fn part2(mut s: &[u8]) -> u64 {
    let mut map = Map::with_capacity_and_hasher(1 << 18, Default::default());
    let mut mask = Mask::default();
    let mut n_ones = 0;
    for instruction in parse_instructions(s) {
        match instruction {
            Instruction::Mask(new_mask) => {
                mask = new_mask;
                n_ones = 1 << mask.mask.count_ones();
            }
            Instruction::Write { addr, value } => {
                let addr = addr | mask.value;
                let mask = mask.mask;
                let mut xor = mask + 1;
                for _ in 0..n_ones {
                    count += 1;
                    xor = (xor - 1) & mask;
                    *map.entry(addr ^ xor).or_default() = value;
                }
            }
        }
    }
    map.values().sum()
}

#[test]
fn test_day14_part1() {
    assert_eq!(part1(input()), 7477696999511);
}

#[test]
fn test_day14_part2() {
    assert_eq!(part2(input()), 3687727854171);
}
