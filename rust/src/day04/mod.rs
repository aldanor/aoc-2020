use memchr::Memchr2;

use crate::utils::*;

#[inline]
fn check_year(mut s: &[u8], lo: u16, hi: u16) -> bool {
    s.len() == 4 && {
        let v = parse_int_fast::<u16>(&mut s, 4, 4);
        v >= lo && v <= hi
    }
}

#[inline]
fn check_height(mut s: &[u8], lo: u16, hi: u16) -> bool {
    s.len() >= 2 && {
        let v = parse_int_fast::<u16>(&mut s, 2, 3);
        v >= lo && v <= hi
    }
}

#[inline]
fn is_valid(k: &[u8], v: &[u8]) -> bool {
    match k {
        b"byr" => check_year(v, 1920, 2002),
        b"iyr" => check_year(v, 2010, 2020),
        b"eyr" => check_year(v, 2020, 2030),
        b"hgt" => {
            let (v, units) = v.split_at(v.len() - 2);
            match units {
                b"cm" => check_height(v, 150, 193),
                b"in" => check_height(v, 59, 76),
                _ => false,
            }
        }
        b"hcl" => v.len() == 7 && v.get_first() == b'#' && v[1..].iter().all(u8::is_ascii_hexdigit),
        b"ecl" => matches!(
            v,
            b"amb" | b"blu" | b"brn" | b"gry" | b"grn" | b"hzl" | b"oth"
        ),
        b"pid" => v.len() == 9 && v.iter().all(u8::is_ascii_digit),
        _ => false,
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u16 {
    let (mut prev, mut valid_passports, mut fields_present) = (0, 0, 0);
    for pos in Memchr2::new(b'\n', b' ', s) {
        if prev == pos {
            valid_passports += (fields_present == 7) as u16;
            fields_present = 0;
        } else {
            fields_present += (s.get_at(prev) != b'c') as u8;
        }
        prev = pos + 1;
    }
    valid_passports
}

#[inline]
pub fn part2(s: &[u8]) -> u16 {
    let (mut prev, mut valid_passports, mut fields_valid) = (0, 0, 0);
    for pos in Memchr2::new(b'\n', b' ', s) {
        if prev == pos {
            valid_passports += (fields_valid == 7) as u16;
            fields_valid = 0;
        } else {
            let (k, v) = (&s[prev..prev + 3], &s[prev + 4..pos]);
            fields_valid += is_valid(k, v) as u8;
        }
        prev = pos + 1;
    }
    valid_passports
}

#[test]
fn test_day04_part1() {
    assert_eq!(part1(input()), 230);
}

#[test]
fn test_day04_part2() {
    assert_eq!(part2(input()), 156);
}
