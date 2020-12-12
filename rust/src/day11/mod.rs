use std::slice::{from_raw_parts, from_raw_parts_mut};

use arrayvec::ArrayVec;
use packed_simd_2::u8x32;

use crate::utils::*;

const MAX_LEN: usize = 1 << 14;
const MAX_GAPS: usize = 1 << 11;
const PAD: usize = 2; // requires 3 for 7x7 conv

#[inline]
fn iter_main_diagonals(
    w: usize,
    h: usize,
) -> impl Iterator<Item = impl Iterator<Item = (usize, usize)>> {
    // iterate over all main diagonals of a rectangle and return coordinates
    (0..w + h - 1).map(move |s| {
        let z1 = s.saturating_sub(w - 1);
        let z2 = s.saturating_sub(h - 1);
        (z1..=s - z2).map(move |j| (s - j, j))
    })
}

#[inline]
fn iter_off_diagonals(
    w: usize,
    h: usize,
) -> impl Iterator<Item = impl Iterator<Item = (usize, usize)>> {
    // iterate over all off diagonals of a rectangle and return coordinates
    (0..w + h - 1).map(move |s| {
        let z1 = s.saturating_sub(w - 1);
        let z2 = s.saturating_sub(h - 1);
        (z1..=s - z2).map(move |j| (w - 1 - (s - j), j))
    })
}

struct State {
    states: ArrayVec<[u8; MAX_LEN]>,
    counts: ArrayVec<[u8; MAX_LEN]>,
    lanes: usize,
    height: usize,
    gaps: ArrayVec<[(usize, usize); MAX_GAPS]>,
}

impl State {
    pub fn parse(mut s: &[u8]) -> Self {
        let real_width = s.memchr(b'\n');
        let lanes = (real_width + 31) / 32;
        let width = lanes * 32 + 2 * PAD;
        let mut height = 0;
        let mut states = ArrayVec::new();
        for _ in 0..width * PAD {
            states.push(16); // first padding rows
        }
        while s.len() > 1 {
            height += 1;
            for _ in 0..PAD {
                states.push(16); // first padding columns
            }
            for i in 0..real_width {
                // store 16 if it's a floor ('.'); 0 otherwise
                states.push((s.get_at(i).wrapping_sub(b'.').eq(&0) as u8).wrapping_shl(4));
            }
            for _ in 0..(width - real_width - PAD) {
                states.push(16); // extended columns + last padding column
            }
            s = s.advance(real_width + 1);
        }
        for _ in 0..width * PAD {
            states.push(16); // last padding rows
        }
        let mut counts = ArrayVec::new();
        unsafe { counts.set_len(states.len()) };
        Self {
            states,
            counts,
            lanes,
            height,
            gaps: ArrayVec::new(),
        }
    }

    #[inline]
    const fn width(&self) -> usize {
        2 * PAD + self.lanes * 32 // row width including pre/post padding
    }

    #[inline]
    fn read_states(&self, pos: usize) -> u8x32 {
        unsafe {
            u8x32::from_slice_unaligned_unchecked(from_raw_parts(self.states.as_ptr().add(pos), 32))
        }
    }

    #[inline]
    fn read_counts(&self, pos: usize) -> u8x32 {
        unsafe {
            u8x32::from_slice_unaligned_unchecked(from_raw_parts(self.counts.as_ptr().add(pos), 32))
        }
    }

    #[inline]
    fn write_states(&mut self, pos: usize, states: u8x32) {
        unsafe {
            states.write_to_slice_unaligned_unchecked(from_raw_parts_mut(
                self.states.as_mut_ptr().add(pos),
                32,
            ))
        }
    }

    #[inline]
    fn write_counts(&mut self, pos: usize, counts: u8x32) {
        unsafe {
            counts.write_to_slice_unaligned_unchecked(from_raw_parts_mut(
                self.counts.as_mut_ptr().add(pos),
                32,
            ))
        }
    }

    pub fn update_counts_1(&mut self) {
        let width = self.width();
        for i in PAD..self.height + PAD {
            let m = i * width;
            let (u, d) = (m - width, m + width);
            for lane in 0..self.lanes {
                let j = PAD + lane * 32;
                let (l, r) = (j - 1, j + 1);
                // zero it out
                let mut counts = u8x32::splat(0);
                // all cells row above
                counts += self.read_states(u + l);
                counts += self.read_states(u + j);
                counts += self.read_states(u + r);
                // left/right mid row
                counts += self.read_states(m + l);
                counts += self.read_states(m + r);
                // all cells row below
                counts += self.read_states(d + l);
                counts += self.read_states(d + j);
                counts += self.read_states(d + r);
                // ignore floors
                counts &= u8x32::splat(0x0f);
                // write it
                self.write_counts(m + j, counts);
            }
        }
    }

    pub fn step_1(&mut self) -> bool {
        self.update_counts_1();
        self.step(4)
    }

    pub fn step_2(&mut self) -> bool {
        self.update_counts_2();
        self.step(5)
    }

    pub fn step(&mut self, threshold: u8) -> bool {
        let width = self.width();
        let mut changed = false;
        for i in PAD..self.height + PAD {
            let m = i * width;
            for lane in 0..self.lanes {
                let j = PAD + lane * 32;
                let k = m + j;
                let counts = self.read_counts(k);
                let old = self.read_states(k);
                let new = old;
                let new = counts.eq(u8x32::splat(0)).select(u8x32::splat(1), new);
                let new = counts
                    .ge(u8x32::splat(threshold))
                    .select(u8x32::splat(0), new);
                let new = old.eq(u8x32::splat(16)).select(u8x32::splat(16), new);
                changed = changed || new.ne(old).any();
                self.write_states(k, new);
            }
        }
        changed
    }

    #[inline]
    fn read2(&self, i1: usize, i2: usize) -> u8x32 {
        // if the near one is non-floor, use that; otherwise use the far one
        let s1 = self.read_states(i1);
        let s2 = self.read_states(i2);
        s1.ne(u8x32::splat(16)).select(s1, s2)
    }

    // #[inline]
    // fn read3(&self, i1: usize, i2: usize, i3: usize) -> u8x32 {
    //     // same as read2(), but uses the 7x7 convolution (3 tiles far)
    //     let s1 = self.read_states(i1);
    //     let s2 = self.read_states(i2);
    //     let floor = u8x32::splat(16);
    //     let s12 = s1.ne(floor).select(s1, s2);
    //     let s3 = self.read_states(i3);
    //     let s123 = s12.ne(floor).select(s12, s3);
    //     s123
    // }

    pub fn update_counts_local(&mut self) {
        let width = self.width();

        for i in PAD..self.height + PAD {
            let m = i * width;
            let (u1, d1) = (m - 1 * width, m + 1 * width);
            let (u2, d2) = (m - 2 * width, m + 2 * width);
            // let (u3, d3) = (m - 3 * width, m + 3 * width);
            for lane in 0..self.lanes {
                let j = PAD + lane * 32;
                let (l1, r1) = (j - 1, j + 1);
                let (l2, r2) = (j - 2, j + 2);
                // let (l3, r3) = (j - 3, j + 3);
                let mut counts = u8x32::splat(0); // zero it out

                // // 7x7
                // counts += self.read3(u1 + l1, u2 + l2, u3 + l3); // up-left
                // counts += self.read3(u1 + j, u2 + j, u3 + j); // up
                // counts += self.read3(u1 + r1, u2 + r2, u3 + r3); // up-right
                // counts += self.read3(m + l1, m + l2, m + l3); // left
                // counts += self.read3(m + r1, m + r2, m + r3); // right
                // counts += self.read3(d1 + l1, d2 + l2, d3 + l3); // down-left
                // counts += self.read3(d1 + j, d2 + j, d3 + j); // down
                // counts += self.read3(d1 + r1, d2 + r2, d3 + r3); // down-right

                // 5x5
                counts += self.read2(u1 + l1, u2 + l2); // up-left
                counts += self.read2(u1 + j, u2 + j); // up
                counts += self.read2(u1 + r1, u2 + r2); // up-right
                counts += self.read2(m + l1, m + l2); // left
                counts += self.read2(m + r1, m + r2); // right
                counts += self.read2(d1 + l1, d2 + l2); // down-left
                counts += self.read2(d1 + j, d2 + j); // down
                counts += self.read2(d1 + r1, d2 + r2); // down-right

                counts &= u8x32::splat(0x0f); // ignore floors
                self.write_counts(m + j, counts); // write it
            }
        }
    }

    #[inline]
    fn record_gaps(&mut self, indices: impl Iterator<Item = usize>) {
        let mut prev = None;
        for (i, index) in indices.enumerate() {
            let v = self.states.get_at(index);
            if v != 16 {
                if let Some((prev_i, prev_index)) = prev {
                    if i - prev_i > 2 {
                        // 3 tiles or more
                        unsafe {
                            self.gaps.push_unchecked((prev_index, index));
                        }
                    }
                }
                prev = Some((i, index));
            }
        }
    }

    pub fn find_gaps(&mut self) {
        // find all pairs of non-floor points that are on the same vertical/horizontal/diagonal
        // line in any of the four directions and have more than one floor between them
        // (and store absolute indices/offsets from the start of the states/counts arrays)
        let width = self.width(); // with padding
        let (x1, x2, y1, y2) = (PAD, width - PAD, PAD, self.height + PAD); // rectangle coords
        for y in y1..y2 {
            self.record_gaps((x1..x2).map(|x| y * width + x)); // horizontal
        }
        for x in x1..x2 {
            self.record_gaps((y1..y2).map(|y| y * width + x)); // vertical
        }
        for d in iter_main_diagonals(x2 - x1, y2 - y1) {
            self.record_gaps(d.map(|(x, y)| (y1 + y) * width + x1 + x)); // main diag
        }
        for d in iter_off_diagonals(x2 - x1, y2 - y1) {
            self.record_gaps(d.map(|(x, y)| (y1 + y) * width + x1 + x)); // off diag
        }
    }

    #[inline]
    pub fn update_counts_2(&mut self) {
        self.update_counts_local();
        for &(i, j) in &self.gaps {
            self.counts.add_at(i, self.states.get_at(j) & 0x0f);
            self.counts.add_at(j, self.states.get_at(i) & 0x0f);
        }
    }

    #[inline]
    pub fn occupied(&self) -> u32 {
        // this is a simple way of doing it, but could use vectorized sum instead
        let width = self.width();
        let mut count = 0;
        for i in PAD..self.height + PAD {
            let m = i * width;
            for lane in 0..self.lanes {
                let j = PAD + lane * 32;
                let states = self.read_states(m + j);
                let occupied = (states & u8x32::splat(0x0f)).ne(u8x32::splat(0));
                count += occupied.bitmask().count_ones();
            }
        }
        count
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u32 {
    assert!(std::is_x86_feature_detected!("avx2"));
    let mut state = State::parse(s);
    while state.step_1() {}
    state.occupied()
}

#[inline]
pub fn part2(s: &[u8]) -> u32 {
    assert!(std::is_x86_feature_detected!("avx2"));
    let mut state = State::parse(s);
    state.find_gaps();
    while state.step_2() {}
    state.occupied()
}

#[test]
fn test_day11_part1() {
    assert_eq!(part1(input()), 2270);
}

#[test]
fn test_day11_part2() {
    assert_eq!(part2(input()), 2042);
}
