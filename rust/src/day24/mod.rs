use std::iter;
use std::slice;

use crate::utils::*;

use packed_simd_2::u8x64;
use rustc_hash::FxHashMap;

type Coord = i16;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
fn parse_coords(s: &[u8]) -> impl Iterator<Item = (Coord, Coord)> + '_ {
    // these are yielded in what's apparently called 'doublewidth coordinate' format
    let (mut x, mut y) = (0, 0);
    let mut s = s;
    iter::from_fn(move || {
        while !s.is_empty() {
            match s.get_first() {
                b'n' => {
                    y += 1;
                    x += 1 - 2 * (s.get_at(1) == b'w') as Coord;
                    s = s.advance(1);
                }
                b's' => {
                    y -= 1;
                    x += 1 - 2 * (s.get_at(1) == b'w') as Coord;
                    s = s.advance(1);
                }
                b'e' => {
                    x += 2;
                }
                b'w' => {
                    x -= 2;
                }
                _ => {
                    let out = (x, y);
                    x = 0;
                    y = 0;
                    s = s.advance(1);
                    return Some(out);
                }
            }
            s = s.advance(1);
        }
        None
    })
}

#[inline]
pub fn part1(s: &[u8]) -> usize {
    let mut counts = FxHashMap::with_capacity_and_hasher(1 << 9, Default::default());
    parse_coords(s).for_each(|coord| *counts.entry(coord).or_default() += 1);
    return counts.values().map(|n| (n % 2 != 0) as usize).sum();
}

type Lane = u8x64; // lane type
const W_LANE: usize = Lane::lanes(); // number of bytes in a lane
const N_LANE: usize = 4; // number of lanes in a row
const N: usize = N_LANE * W_LANE; // width / height, without padding
const W: usize = N + 2; // total row height, including padding

#[inline]
fn splat(x: u8) -> Lane {
    Lane::splat(x)
}

#[inline]
fn doublewidth_to_oddr((x_dw, y_dw): (Coord, Coord)) -> (Coord, Coord) {
    // double-width -> cube
    let x_cube = (x_dw - y_dw) / 2;
    let z_cube = y_dw;
    // cube -> oddr
    let x_oddr = x_cube + (z_cube - (z_cube & 1)) / 2;
    let y_oddr = z_cube;
    (x_oddr, y_oddr)
}

#[derive(Clone, Debug)]
struct Grid {
    states: [u8; W * W],
    counts: [u8; W * W],
    y_bounds: (usize, usize),
}

impl Grid {
    pub fn new(coords: impl IntoIterator<Item = (Coord, Coord)>) -> Self {
        let coords = coords.into_iter().collect::<Vec<_>>();
        let (xmin, xmax, ymin, ymax) = coords.iter().fold(
            (Coord::MAX, Coord::MIN, Coord::MAX, Coord::MIN),
            |(xmin, xmax, ymin, ymax), &(x, y)| {
                (xmin.min(x), xmax.max(x), ymin.min(y), ymax.max(y))
            },
        );
        let (x0, y0) = (
            -(xmin + xmax) / 2 + W as Coord / 2,
            -(ymin + ymax) / 2 + W as Coord / 2,
        );
        let (x0, y0) = (x0 - (x0 & 1), y0 - (y0 & 1)); // careful to retain row parity
        let mut states = [0u8; W * W];
        for &(x, y) in &coords {
            let offset = (y + y0) as usize * W + (x + x0) as usize;
            states[offset] = (states[offset] + 1) & 1;
        }
        let y_bounds = ((ymin + y0) as usize, (ymax + y0) as usize);
        let counts = [0u8; W * W];
        Self {
            states,
            counts,
            y_bounds,
        }
    }

    #[inline]
    pub fn read_states(&self, pos: usize) -> Lane {
        unsafe {
            let slice = slice::from_raw_parts(self.states.as_ptr().add(pos), W_LANE);
            Lane::from_slice_unaligned_unchecked(slice)
        }
    }

    #[inline]
    pub fn read_counts(&self, pos: usize) -> Lane {
        unsafe {
            let slice = slice::from_raw_parts(self.counts.as_ptr().add(pos), W_LANE);
            Lane::from_slice_unaligned_unchecked(slice)
        }
    }

    #[inline]
    fn write_states(&mut self, pos: usize, states: Lane) {
        unsafe {
            let slice = slice::from_raw_parts_mut(self.states.as_mut_ptr().add(pos), W_LANE);
            states.write_to_slice_unaligned_unchecked(slice);
        }
    }

    #[inline]
    fn write_counts(&mut self, pos: usize, counts: Lane) {
        unsafe {
            let slice = slice::from_raw_parts_mut(self.counts.as_mut_ptr().add(pos), W_LANE);
            counts.write_to_slice_unaligned_unchecked(slice);
        }
    }

    #[inline]
    pub fn update_counts(&mut self) {
        let (ymin, ymax) = self.y_bounds;
        for y in (ymin - 1)..=(ymax + 1) {
            let row_parity = y & 1;
            let y = y * W;
            let (u, d) = (y - W, y + W);
            for lane in 0..N_LANE {
                let x = 1 + lane * W_LANE;
                let (l, r) = (x - 1, x + 1);
                let mut counts = splat(0);
                if row_parity == 0 {
                    counts += self.read_states(r + y);
                    counts += self.read_states(x + d);
                    counts += self.read_states(l + d);
                    counts += self.read_states(l + y);
                    counts += self.read_states(l + u);
                    counts += self.read_states(x + u);
                } else {
                    counts += self.read_states(r + y);
                    counts += self.read_states(r + d);
                    counts += self.read_states(x + d);
                    counts += self.read_states(l + y);
                    counts += self.read_states(x + u);
                    counts += self.read_states(r + u);
                }
                self.write_counts(x + y, counts);
            }
        }
    }

    #[inline]
    pub fn update_states(&mut self) {
        let (ymin, ymax) = self.y_bounds;
        for y in (ymin - 1)..=(ymax + 1) {
            let y = y * W;
            for lane in 0..N_LANE {
                let pos = y + (1 + lane * W_LANE);
                let counts = self.read_counts(pos);
                let old = self.read_states(pos);
                let new = counts.eq(splat(2)) | (old.eq(splat(1)) & counts.eq(splat(1)));
                self.write_states(pos, new.select(splat(1), splat(0)))
            }
        }
    }

    #[inline]
    pub fn step(&mut self) {
        self.update_counts();
        self.update_states();
        self.y_bounds = (self.y_bounds.0 - 1, self.y_bounds.1 + 1);
    }

    #[inline]
    pub fn count(&self) -> usize {
        let mut count = 0;
        let (ymin, ymax) = self.y_bounds;
        for y in ymin..=ymax {
            let y = y * W;
            for lane in 0..N_LANE {
                count += self.read_states(y + (1 + lane * W_LANE)).wrapping_sum() as usize;
            }
        }
        count
    }
}

#[inline]
pub fn part2(s: &[u8]) -> usize {
    let mut grid = Grid::new(parse_coords(s).map(doublewidth_to_oddr));
    for _ in 0..100 {
        grid.step();
    }
    grid.count()
}

#[test]
fn test_day24_part1() {
    assert_eq!(part1(input()), 341);
}

#[test]
fn test_day24_part2() {
    assert_eq!(part2(input()), 3700);
}
