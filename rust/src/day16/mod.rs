use std::iter;
use std::ops::RangeInclusive;

use arrayvec::ArrayVec;

use crate::utils::*;

const MAX_FIELDS: usize = 32;
type Ticket = ArrayVec<[u16; MAX_FIELDS]>;

// Workaround for a bug in lifetimes with impl traits:
// https://github.com/rust-lang/rust/issues/61756
pub trait Captures<'a> {}
impl<'a, T: ?Sized> Captures<'a> for T {}

#[inline]
fn parse_valid_ranges<'a, 'b>(
    s: &'a mut &'b [u8],
) -> impl Iterator<Item = [RangeInclusive<u16>; 2]> + 'a + Captures<'b> {
    iter::from_fn(move || {
        if s.get_first() != b'\n' {
            *s = s.skip_past(b':', 1);
            let s0 = parse_int_fast(s, 1, 3);
            let e0 = parse_int_fast(s, 1, 3);
            *s = s.advance(3);
            let s1 = parse_int_fast(s, 1, 3);
            let e1 = parse_int_fast(s, 1, 3);
            Some([s0..=e0, s1..=e1])
        } else {
            None
        }
    })
}

#[inline]
fn parse_ticket(s: &mut &[u8], n: usize) -> Ticket {
    let mut ticket = Ticket::new();
    for _ in 0..n {
        unsafe { ticket.push_unchecked(parse_int_fast(s, 1, 3)) };
    }
    ticket
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> u16 {
    let mut mask = [0u16; 1024];
    let mut n_fields = 0;
    for ranges in parse_valid_ranges(&mut s) {
        for range in &ranges {
            for index in range.clone() {
                mask.set_at(index as _, index as _);
            }
        }
        n_fields += 1;
    }
    s = s.skip_past(b':', 1);
    s = s.skip_past(b':', 1);
    let mut error_rate = 0u16;
    while s.len() > 1 {
        for &field in &parse_ticket(&mut s, n_fields) {
            error_rate = error_rate.wrapping_add(field ^ mask.get_at(field as _));
        }
    }
    error_rate
}

#[inline]
pub fn part2(mut s: &[u8]) -> u64 {
    let mut mask = [0u32; 1024];
    let mut n_fields = 0;
    for (i, ranges) in parse_valid_ranges(&mut s).enumerate() {
        let j = 1 << i;
        for range in &ranges {
            for index in range.clone() {
                *mask.get_mut_at(index as _) |= j;
            }
        }
        n_fields += 1;
    }
    s = s.skip_past(b':', 1);
    let our_ticket = parse_ticket(&mut s, n_fields);
    s = s.skip_past(b':', 1);
    let mut fields = ArrayVec::<[u16; MAX_FIELDS]>::new();
    let mut map = ArrayVec::<[u32; MAX_FIELDS]>::new();
    for _ in 0..n_fields {
        map.push((1 << n_fields) - 1);
    }
    'main: while s.len() > 1 {
        fields.clear();
        for &field in &parse_ticket(&mut s, n_fields) {
            unsafe { fields.push_unchecked(field) };
            if mask.get_at(field as _) == 0 {
                continue 'main;
            }
        }
        for (i, &field) in fields.iter().enumerate() {
            *map.get_mut_at(i) &= mask.get_at(field as _);
        }
    }
    let mut tagged_map = ArrayVec::<[(u32, u32); MAX_FIELDS]>::new();
    for i in 0..n_fields {
        tagged_map.push((i as _, map[i]));
    }
    quickersort::sort_by_key(&mut tagged_map, |&(_, m)| m.count_ones());
    let mut exclude = !0;
    let mut answer = 1;
    for &(i, choices) in &tagged_map {
        let choices = choices & exclude;
        debug_assert_eq!(choices.count_ones(), 1);
        let k = choices.trailing_zeros();
        if k < 6 {
            answer *= our_ticket.get_at(i as _) as u64;
        }
        exclude &= !choices;
    }
    answer
}

#[test]
fn test_day16_part1() {
    assert_eq!(part1(input()), 23044);
}

#[test]
fn test_day16_part2() {
    assert_eq!(part2(input()), 3765150732757);
}
