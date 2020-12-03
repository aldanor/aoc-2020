use std::borrow::Cow;

use arrayvec::ArrayVec;

const MAX_WIDTH: usize = 32;
const MAX_DX: usize = 10;
const MAX_DY: usize = 4;

#[derive(Debug, Default, Clone)]
struct Cycle {
    pub len: usize,
    pub steps: ArrayVec<[u8; MAX_WIDTH]>,
}

impl Cycle {
    fn new(w: usize, dx: usize, dy: usize) -> Self {
        // element `i` holds the offset to step from item `i` to item `i + 1`
        // the number of steps is exactly `w` (it's guaranteed to cycle after
        // that, but it may contain sub-cycles, e.g. w=10, dx=5, that's ok).
        if w == 0 || dx == 0 || dy == 0 {
            return Default::default();
        }
        let (mut x, mut prev) = (0, 0);
        let mut steps = ArrayVec::new();
        for i in 0..w {
            x = (x + dx) % w;
            let pos = (i + 1) * dy * (w + 1) + x;
            steps.push((pos - prev) as _);
            prev = pos;
        }
        Self { steps, len: prev }
    }

    #[inline]
    fn get(w: usize, dx: usize, dy: usize) -> Cow<'static, Self> {
        if w < MAX_WIDTH && dx < MAX_DX && dy < MAX_DY {
            Cow::Borrowed(&BAKERY[w * MAX_DX * MAX_DY + dx * MAX_DY + dy])
        } else {
            Cow::Owned(Self::new(w, dx, dy))
        }
    }

    #[inline]
    fn eval(&self, s: &[u8]) -> u8 {
        let n_full_cycles = s.len() / self.len;
        let mut p = s.as_ptr();
        let mut count = 0;
        unsafe {
            for _ in 0..n_full_cycles {
                for j in 0..self.steps.len() {
                    count += (*p == b'#') as u8;
                    p = p.add(*self.steps.get_unchecked(j) as usize);
                }
            }
            let (p_end, mut j) = (s.as_ptr().add(s.len()), 0);
            while p < p_end && j < self.steps.len() {
                count += (*p == b'#') as u8;
                p = p.add(*self.steps.get_unchecked(j) as usize);
                j += 1;
            }
        }
        count
    }
}

#[ctor::ctor]
static BAKERY: Vec<Cycle> = {
    let mut out = Vec::new();
    for w in 0..MAX_WIDTH {
        for dx in 0..MAX_DX {
            for dy in 0..MAX_DY {
                out.push(Cycle::new(w, dx, dy));
            }
        }
    }
    out
};

#[inline]
pub fn input() -> &'static [u8] {
    static INPUT: &str = include_str!("input.txt");
    INPUT.as_bytes()
}

pub fn part1(s: &[u8]) -> u8 {
    let w = memchr::memchr(b'\n', s).unwrap();
    return Cycle::get(w, 3, 1).eval(s);
}

pub fn part2(s: &[u8]) -> u32 {
    let w = memchr::memchr(b'\n', s).unwrap();
    (Cycle::get(w, 1, 1).eval(s) as u32)
        * (Cycle::get(w, 3, 1).eval(s) as u32)
        * (Cycle::get(w, 5, 1).eval(s) as u32)
        * (Cycle::get(w, 7, 1).eval(s) as u32)
        * (Cycle::get(w, 1, 2).eval(s) as u32)
}

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

pub fn part1_slow(s: &[u8]) -> u8 {
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

pub fn part2_slow(s: &[u8]) -> u32 {
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
