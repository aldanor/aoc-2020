#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u16 {
    let mut prev = b'\n';
    let mut group = 0u32;
    let mut sum = 0u16;
    for &c in s {
        let c = c ^ b'\n';
        if c != 0 {
            group |= 1u32 << (c & 0x1f)
        } else if prev == 0 {
            sum = sum.wrapping_add(group.count_ones() as u16);
            group = 0;
        }
        prev = c;
    }
    sum
}

#[inline]
pub fn part2(s: &[u8]) -> u16 {
    let mut prev = b' ';
    let mut person = 0;
    let mut group = u32::MAX;
    let mut sum = 0u16;
    for &c in s {
        let c = c ^ b'\n';
        if c != 0 {
            person |= 1u32 << (c & 0x1f);
        } else if prev != 0 {
            group &= person;
            person = 0;
        } else {
            sum = sum.wrapping_add(group.count_ones() as _);
            person = 0;
            group = u32::MAX;
        }
        prev = c;
    }
    sum
}

#[test]
fn test_day06_part1() {
    assert_eq!(part1(input()), 7120);
}

#[test]
fn test_day06_part2() {
    assert_eq!(part2(input()), 3570);
}
