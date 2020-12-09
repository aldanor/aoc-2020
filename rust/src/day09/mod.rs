use arrayvec::ArrayVec;

use crate::utils::*;

type T = i64;
const N: usize = 25;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> T {
    unsafe {
        let mut a = ArrayVec::<[T; 1024]>::new();
        for _ in 0..N {
            let x = parse_int_fast(&mut s, 1, 14);
            a.push_unchecked(x);
        }
        let mut p = a.as_ptr();
        'next: loop {
            let x: T = parse_int_fast(&mut s, 1, 14);
            for i in 0..(N - 1) {
                let v = x.wrapping_sub(*p.add(i));
                for j in i + 1..N {
                    if *p.add(j) == v {
                        a.push_unchecked(x);
                        p = p.add(1);
                        continue 'next;
                    }
                }
            }
            break x;
        }
    }
}

#[inline]
pub fn part2(mut s: &[u8]) -> T {
    let mut next_num = move || parse_int_fast::<T>(&mut s, 1, 14);
    let target: T = *PART1_ANSWER;
    let min = target / 1000;
    let mut x = next_num();
    while x < min {
        x = next_num();
    }
    let mut a = ArrayVec::<[T; 1024]>::new();
    a.push(x);
    let mut sum = x - target;
    unsafe {
        let mut p = a.as_ptr();
        loop {
            while sum < 0 {
                let x = next_num();
                a.push_unchecked(x);
                sum += x;
            }
            if sum == 0 {
                let start = p.offset_from(a.as_ptr()) as usize;
                let (min, max) = &a[start..]
                    .iter()
                    .fold((T::MAX, T::MIN), |(min, max), &x| (min.min(x), max.max(x)));
                break min + max;
            }
            sum -= *p;
            p = p.add(1);
        }
    }
}

#[ctor::ctor]
static PART1_ANSWER: T = part1(input());

#[test]
fn test_day09_part1() {
    assert_eq!(part1(input()), 50047984);
}

#[test]
fn test_day09_part2() {
    assert_eq!(part2(input()), 5407707);
}
