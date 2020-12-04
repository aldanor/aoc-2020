#[inline]
pub fn input() -> &'static [u8] {
    static INPUT: &str = include_str!("input.txt");
    INPUT.as_bytes()
}

pub fn part1(s: &[u8]) -> u16 {
    let (mut prev, mut valid_passports, mut fields_present) = (0, 0, 0);
    for pos in memchr::Memchr2::new(b'\n', b' ', s) {
        if prev == pos {
            valid_passports += (fields_present == 7) as u16;
            fields_present = 0;
        } else {
            fields_present += (unsafe { *s.get_unchecked(prev) } != b'c') as u8;
        }
        prev = pos + 1;
    }
    valid_passports
}

fn check_year(s: &[u8], lo: u16, hi: u16) -> bool {
    // yes, we could use `u16::from_str()`... but this is faster
    s.len() == 4 && {
        let c0 = unsafe { *s.get_unchecked(0) };
        let c1 = unsafe { *s.get_unchecked(1) };
        let c2 = unsafe { *s.get_unchecked(2) };
        let c3 = unsafe { *s.get_unchecked(3) };
        (c0.is_ascii_digit() && c1.is_ascii_digit() && c1.is_ascii_digit() && c2.is_ascii_digit())
            && {
                let v = ((c0 - b'0') as u16) * 1000
                    + ((c1 - b'0') as u16) * 100
                    + ((c2 - b'0') as u16) * 10
                    + ((c3 - b'0') as u16);
                v >= lo && v <= hi
            }
    }
}

fn check_height(s: &[u8], lo: u16, hi: u16) -> bool {
    // yes, we could use `u16::from_str()`... but this is faster
    s.len() >= 2 && {
        let c0 = unsafe { *s.get_unchecked(0) };
        let c1 = unsafe { *s.get_unchecked(1) };
        (c0.is_ascii_digit() && c1.is_ascii_digit()) && {
            let v = ((c0 - b'0') as u16) * 10 + ((c1 - b'0') as u16);
            match s.len() {
                3 => {
                    let c2 = unsafe { *s.get_unchecked(2) };
                    c2.is_ascii_digit() && {
                        let v = v * 10 + ((c2 - b'0') as u16);
                        v >= lo && v <= hi
                    }
                }
                2 => v >= lo && v <= hi,
                _ => false,
            }
        }
    }
}

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
        b"hcl" => {
            v.len() == 7
                && unsafe { *v.get_unchecked(0) == b'#' }
                && v[1..].iter().all(u8::is_ascii_hexdigit)
        }
        b"ecl" => match v {
            b"amb" | b"blu" | b"brn" | b"gry" | b"grn" | b"hzl" | b"oth" => true,
            _ => false,
        },
        b"pid" => v.len() == 9 && v.iter().all(u8::is_ascii_digit),
        _ => false,
    }
}

pub fn part2(s: &[u8]) -> u16 {
    let (mut prev, mut valid_passports, mut fields_valid) = (0, 0, 0);
    for pos in memchr::Memchr2::new(b'\n', b' ', s) {
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
