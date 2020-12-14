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

#[derive(Copy, Clone, Debug)]
enum Cell {
    Empty,
    Single(Mask, u64),
    Multi,
}

#[inline]
pub fn part2(s: &[u8]) -> u64 {
    const MASK_LO: u64 = 0x000000000000FFFF;
    const MASK_HI: u64 = 0xFFFFFFFFFFFF0000;

    let mut map = Map::with_capacity_and_hasher(1 << 18, Default::default());
    let mut update_map = |mask: &Mask, value: u64, idx_lo: usize| {
        let mask_hi = mask.mask & MASK_HI;
        let addr = (idx_lo as u64) | (mask.value & MASK_HI);
        let mut xor = mask_hi + 1;
        for _ in 0..(1 << mask_hi.count_ones()) {
            xor = (xor - 1) & mask_hi;
            *map.entry(addr ^ xor).or_default() = value;
        }
    };

    let mut mask_now = Mask::default();
    let mut cells = [Cell::Empty; 65536];

    for instruction in parse_instructions(s) {
        match instruction {
            Instruction::Mask(mask_new) => mask_now = mask_new,
            Instruction::Write { addr, value } => {
                let mask_lo = (mask_now.mask & MASK_LO) as u16;
                let addr_lo = ((addr | mask_now.value) & MASK_LO) as u16;
                let mut xor_lo = mask_lo + 1;
                for _ in 0..(1 << mask_lo.count_ones()) {
                    xor_lo = (xor_lo - 1) & mask_lo;
                    let idx_lo = (addr_lo ^ xor_lo) as usize;
                    cells[idx_lo] = match cells[idx_lo] {
                        Cell::Empty => Cell::Single(mask_now, value),
                        Cell::Single(mask_old, value_old) => {
                            update_map(&mask_old, value_old, idx_lo);
                            update_map(&mask_now, value, idx_lo);
                            Cell::Multi
                        }
                        Cell::Multi => {
                            update_map(&mask_now, value, idx_lo);
                            Cell::Multi
                        }
                    };
                }
            }
        }
    }

    let sum_multi = map.values().sum::<u64>();
    let sum_single = cells
        .iter()
        .filter_map(|cell| {
            if let Cell::Single(ref mask, value) = cell {
                Some(value * (1 << (mask.mask & MASK_HI).count_ones()))
            } else {
                None
            }
        })
        .sum::<u64>();
    sum_multi + sum_single
}

#[test]
fn test_day14_part1() {
    assert_eq!(part1(input()), 7477696999511);
}

#[test]
fn test_day14_part2() {
    assert_eq!(part2(input()), 3687727854171);
}
