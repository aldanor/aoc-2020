#[derive(Debug, Copy, Clone)]
struct TreeCounter {
    pub dx: u8,
    pub dy: u8,
    pub x: u8,
    pub n: u8,
}

impl TreeCounter {
    pub fn new(dx: u8, dy: u8) -> Self {
        let (x, n) = (0, 0);
        Self { dx, dy, x, n }
    }

    #[inline(always)]
    pub fn process(&mut self, y: u8, line: &[u8], w: u8) {
        if y % self.dy == 0 {
            self.n += (unsafe { *line.get_unchecked(self.x as usize) } == b'#') as u8;
            self.x = (self.x + self.dx) % w;
        }
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    static INPUT: &str = include_str!("input.txt");
    INPUT.as_bytes()
}

pub fn part1(s: &[u8]) -> u8 {
    let mut c = TreeCounter::new(3, 1);
    let (mut y, mut pos) = (0, 0);
    let w = memchr::memchr(b'\n', s).unwrap();
    while pos + 1 < s.len() {
        let s = &s[pos..pos + w];
        c.process(y, s, w as _);
        pos += w + 1;
        y = y.wrapping_add(1);
    }
    c.n
}

pub fn part2(s: &[u8]) -> u32 {
    let mut c0 = TreeCounter::new(1, 1);
    let mut c1 = TreeCounter::new(3, 1);
    let mut c2 = TreeCounter::new(5, 1);
    let mut c3 = TreeCounter::new(7, 1);
    let mut c4 = TreeCounter::new(1, 2);
    let (mut y, mut pos) = (0, 0);
    let w = memchr::memchr(b'\n', s).unwrap();
    while pos + 1 < s.len() {
        let s = &s[pos..pos + w];
        c0.process(y, s, w as _);
        c1.process(y, s, w as _);
        c2.process(y, s, w as _);
        c3.process(y, s, w as _);
        c4.process(y, s, w as _);
        pos += w + 1;
        y = y.wrapping_add(1);
    }
    (c0.n as u32) * (c1.n as u32) * (c2.n as u32) * (c3.n as u32) * (c4.n as u32)
}
