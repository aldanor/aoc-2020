use crate::utils::*;

#[derive(Clone, Debug, Default)]
pub struct Password<'a> {
    pub pass: &'a [u8],
    pub n1: u8,
    pub n2: u8,
    pub char: u8,
}

impl<'a> Password<'a> {
    #[inline]
    pub fn parse(s: &mut &'a [u8]) -> Self {
        let n1 = parse_int_fast(s, 1, 2);
        let n2 = parse_int_fast(s, 1, 2);
        let char = s.get_first();
        *s = s.advance(3);
        let i = s.memchr(b'\n');
        let pass = &s[..i];
        *s = s.advance(i + 1);
        Self { pass, n1, n2, char }
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> u16 {
    let mut valid = 0u16;
    while s.len() > 1 {
        let p = Password::parse(&mut s);
        let n = p
            .pass
            .into_iter()
            .take(32)
            .map(|&c| (c == p.char) as u8)
            .sum::<u8>();
        valid = valid.wrapping_add((n >= p.n1 && n <= p.n2) as u16);
    }
    valid
}

#[inline]
pub fn part2(mut s: &[u8]) -> u16 {
    let mut valid = 0;
    while s.len() > 1 {
        let p = Password::parse(&mut s);
        let match1 = p.pass.get_at((p.n1 - 1) as usize) == p.char;
        let match2 = p.pass.get_at((p.n2 - 1) as usize) == p.char;
        valid += (match1 != match2) as u16;
    }
    valid
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 477);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 686);
}
