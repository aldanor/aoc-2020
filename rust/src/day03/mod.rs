use std::borrow::Cow;

use arrayvec::ArrayVec;

use crate::utils::*;

const MAX_WIDTH: usize = 32;
const MAX_DX: usize = 10;
const MAX_DY: usize = 4;

#[derive(Debug, Default, Clone)]
struct Cycle {
    pub len: usize,
    pub steps: ArrayVec<[u16; MAX_WIDTH * 4]>,
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
        for i in 0..w * 4 {
            x = (x + dx) % w;
            let pos = (i + 1) * dy * (w + 1) + x;
            steps.push(prev as _);
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

        let f = move |k: usize| unsafe { *self.steps.get_unchecked(k) };
        let g = |p: *const u8, i: u16| unsafe { (*p.add(i as usize) == b'#') as u8 };

        const BATCH_SIZE: usize = 8;
        let n_batches = self.steps.len() / BATCH_SIZE;
        let rem_start = n_batches * BATCH_SIZE;

        let mut p = s.as_ptr();
        let mut count = 0;
        unsafe {
            for _ in 0..n_full_cycles {
                let mut k = 0;
                for _ in 0..n_batches {
                    let (i0, i1, i2, i3) = (f(k + 0), f(k + 1), f(k + 2), f(k + 3));
                    let (i4, i5, i6, i7) = (f(k + 4), f(k + 5), f(k + 6), f(k + 7));
                    let (c0, c1, c2, c3) = (g(p, i0), g(p, i1), g(p, i2), g(p, i3));
                    let (c4, c5, c6, c7) = (g(p, i4), g(p, i5), g(p, i6), g(p, i7));
                    count += c0 + c1 + c2 + c3;
                    count += c4 + c5 + c6 + c7;
                    k += BATCH_SIZE;
                }
                for j in rem_start..self.steps.len() {
                    count += g(p, f(j));
                }
                p = p.add(self.len);
            }
            let (p_end, mut j) = (s.as_ptr().add(s.len()), 0);
            while p < p_end && j < self.steps.len() {
                count += g(p, f(j));
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
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u8 {
    let w = s.memchr(b'\n');
    return Cycle::get(w, 3, 1).eval(s);
}

#[inline]
pub fn part2(s: &[u8]) -> u32 {
    let w = s.memchr(b'\n');
    (Cycle::get(w, 1, 1).eval(s) as u32)
        * (Cycle::get(w, 3, 1).eval(s) as u32)
        * (Cycle::get(w, 5, 1).eval(s) as u32)
        * (Cycle::get(w, 7, 1).eval(s) as u32)
        * (Cycle::get(w, 1, 2).eval(s) as u32)
}

pub mod slow {
    use crate::utils::*;

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
    pub fn part1(s: &[u8]) -> u8 {
        let mut c = TreeCounter::new(3, 1);
        let (mut y, mut pos) = (0, 0);
        let w = s.memchr(b'\n');
        while pos + 1 < s.len() {
            let s = &s[pos..pos + w];
            c.process(y, s, w as _);
            pos += w + 1;
            y = y.wrapping_add(1);
        }
        c.n
    }

    #[inline]
    pub fn part2(s: &[u8]) -> u32 {
        let mut c0 = TreeCounter::new(1, 1);
        let mut c1 = TreeCounter::new(3, 1);
        let mut c2 = TreeCounter::new(5, 1);
        let mut c3 = TreeCounter::new(7, 1);
        let mut c4 = TreeCounter::new(1, 2);
        let (mut y, mut pos) = (0, 0);
        let w = s.memchr(b'\n');
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
}

#[test]
fn test_day03_part1() {
    assert_eq!(part1(input()), 195);
}

#[test]
fn test_day03_part2() {
    assert_eq!(part2(input()), 3772314000);
}
