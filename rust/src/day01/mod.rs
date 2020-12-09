use arrayvec::ArrayVec;

use crate::utils::*;

#[inline]
fn find_sum_min2(s: &[i16]) -> i16 {
    let (mut a, mut b) = (i16::MAX, i16::MAX);
    for x in s {
        if *x >= b {
            // most likely
            continue;
        } else if *x >= a {
            // less likely
            b = *x;
        } else {
            // least likely
            b = a;
            a = *x;
        }
    }
    a + b
}

#[inline]
pub fn parse_input(mut s: &[u8]) -> ArrayVec<[i16; 256]> {
    let mut vec = ArrayVec::new();
    while s.len() > 1 {
        let v = parse_int_fast(&mut s, 1, 4);
        unsafe { vec.push_unchecked(v) };
    }
    vec
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u32 {
    let s = parse_input(s);
    let mut arr = [0i16; 4096];
    for &x in &s {
        let x = x.min(2020) as usize;
        let rem = 2020 - x;
        arr.set_at(rem, x as _);
        let y = arr.get_at(x);
        if y != 0 {
            return (x as u32) * (y as u32);
        }
    }
    0
}

#[inline]
pub fn part2(s: &[u8]) -> u32 {
    let s = parse_input(s);
    let max = 2020 - find_sum_min2(&s);
    let mut arr = ArrayVec::<[i16; 256]>::new();
    for &x in &s {
        if x <= max {
            unsafe { arr.push_unchecked(x) };
        }
    }
    quickersort::sort(&mut arr);
    let n = arr.len();

    for i in 0..n - 2 {
        let a_i = arr.get_at(i);
        let ai_rem = a_i - 2020;
        for j in i + 1..n - 1 {
            let a_j = arr.get_at(j);
            let a_i_j = ai_rem + a_j;
            for k in j + 1..n {
                let a_k = arr.get_at(k);
                let a_i_j_k = a_i_j + a_k;
                if a_i_j_k < 0 {
                    continue;
                } else if a_i_j_k > 0 {
                    break;
                } else {
                    return (a_i as u32) * (a_j as u32) * (a_k as u32);
                }
            }
        }
    }
    0
}

#[test]
fn test_day01_part1() {
    assert_eq!(part1(input()), 974304);
}

#[test]
fn test_day01_part2() {
    assert_eq!(part2(input()), 236430480);
}
