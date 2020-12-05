#[inline]
pub fn input() -> &'static [u8] {
    static INPUT: &str = include_str!("input.txt");
    INPUT.as_bytes()
}

#[inline(always)]
unsafe fn mangle(s: &[u8]) -> u64 {
    let mut hi = u64::from_be_bytes(*s.as_ptr().cast());
    let last = &mut hi as *mut _ as *mut u8; // le/x86
    let lo1 = s.get_unchecked(8);
    let lo2 = s.get_unchecked(9);
    /*
    Byte codes for L and R are 76 and 82 respectively. Since we're searching
    for minimum, we need to flip it which we'll do by xoring with 0xff at the end.
    We would also like to pack the three last L/R bytes into the lowest byte.
    - 76 = 0b10011000
    - 82 = 0b10100100
    Note if we have three bytes B0, B1, B2 each equal to either L/R, then
    (B0 << 1) | B1 | (B2 >> 4) will take on 8 different values, and hence
    it's decodable back. Moreover, the joint form retains sorting order:
    {
        'LLL': 220,
        'LLR': 221,
        'LRL': 222,
        'LRR': 223,
        'RLL': 236,
        'RLR': 237,
        'RRL': 246,
        'RRR': 247,
    }
     */
    *last = ((*last << 1) | lo1 | (lo2 >> 4)) ^ 0xff;
    hi
}

#[inline]
unsafe fn unmangle(m: u64) -> u16 {
    let mut a: [u8; 10] = Default::default();
    *a.as_mut_ptr().cast() = m.to_be_bytes();
    let lo = match a[7] ^ 0xff {
        220 => b"LLL",
        221 => b"LLR",
        222 => b"LRL",
        223 => b"LRR",
        236 => b"RLL",
        237 => b"RLR",
        246 => b"RRL",
        _ => b"RRR", // 247
    };
    a[7] = lo[0];
    a[8] = lo[1];
    a[9] = lo[2];
    decode(&a)
}

#[inline]
unsafe fn decode(s: &[u8]) -> u16 {
    // stagger the loads
    let v0 = ((*s.get_unchecked(0) == b'B') as u16) << 9;
    let v1 = ((*s.get_unchecked(1) == b'B') as u16) << 8;
    let v2 = ((*s.get_unchecked(2) == b'B') as u16) << 7;
    let v3 = ((*s.get_unchecked(3) == b'B') as u16) << 6;
    let x0 = v0 | v1 | v2 | v3;
    let v4 = ((*s.get_unchecked(4) == b'B') as u16) << 5;
    let v5 = ((*s.get_unchecked(5) == b'B') as u16) << 4;
    let v6 = ((*s.get_unchecked(6) == b'B') as u16) << 3;
    let x1 = v4 | v5 | v6;
    let v7 = ((*s.get_unchecked(7) == b'R') as u16) << 2;
    let v8 = ((*s.get_unchecked(8) == b'R') as u16) << 1;
    let v9 = ((*s.get_unchecked(9) == b'R') as u16) << 0;
    let x2 = v7 | v8 | v9;
    x0 | x1 | x2
}

pub fn part1(s: &[u8]) -> u16 {
    let r = s
        .chunks(11)
        .take(s.len() / 11)
        .fold(u64::MAX, |m, s| m.min(unsafe { mangle(s) }));
    unsafe { unmangle(r) }
}

pub fn part2(s: &[u8]) -> u16 {
    // TODO
    0
}
