#[inline]
pub fn input() -> &'static [u8] {
    static INPUT: &[u8] = include_bytes!("input.txt");
    INPUT
}

pub fn part1(s: &[u8]) -> u16 {
    let mut prev = b'\n';
    let mut group = 0u32;
    let mut sum = 0;
    for &c in s {
        let c = c.wrapping_sub(b'\n');
        if c != 0 {
            group |= 1u32 << c.wrapping_sub(b'a' - b'\n');
        } else if prev == 0 {
            sum += group.count_ones() as u16;
            group = 0;
        }
        prev = c;
    }
    sum
}

pub fn part2(s: &[u8]) -> u16 {
    let mut prev = b' ';
    let mut person = 0;
    let mut group = u32::MAX;
    let mut sum = 0;
    for &c in s {
        let c = c.wrapping_sub(b'\n');
        if c != 0 {
            person |= 1u32 << c.wrapping_sub(b'a' - b'\n');
        } else if prev != 0 {
            group &= person;
            person = 0;
        } else {
            sum += group.count_ones() as u16;
            person = 0;
            group = u32::MAX;
        }
        prev = c;
    }
    sum
}
