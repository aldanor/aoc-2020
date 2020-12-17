use arrayvec::ArrayVec;
use rustc_hash::FxHashSet;

use crate::utils::*;

const N_ITER: usize = 6;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
fn parse_board(mut s: &[u8]) -> ArrayVec<[(i32, i32); 256]> {
    let mut out = ArrayVec::new();
    let width = s.memchr(b'\n');
    let mut row = 0;
    while s.len() > 1 {
        for col in 0..width {
            if s.get_at(col) == b'#' {
                unsafe { out.push_unchecked((row, col as _)) };
            }
        }
        row += 1;
        s = s.advance(width + 1);
    }
    out
}

#[inline]
pub fn part1(s: &[u8]) -> u64 {
    // ones coordinate fits within 0-16, the other two fit within 0-32 => 14 bits
    const D0: usize = 1 << 4;
    const D1: usize = 1 << 5;
    const D2: usize = 1 << 5;

    // total maximum grid size
    const BOARD_SIZE: usize = D0 * D1 * D2;

    // max number of active cells
    const MAX_ACTIVE: usize = 1 << 11;

    // absolute offsets in each direction
    type Offset = i16;
    const D0L: Offset = -1 * (D1 as Offset) * (D2 as Offset);
    const D0R: Offset = 1 * (D1 as Offset) * (D2 as Offset);
    const D1L: Offset = -1 * (D2 as Offset);
    const D1R: Offset = 1 * (D2 as Offset);
    const D2L: Offset = -1;
    const D2R: Offset = 1;

    // total number of neighbors
    const N_NEIGHBORS: usize = 26;

    // each of the 26 neighbors in absolute terms
    const OFFSETS: [Offset; N_NEIGHBORS] = [
        D0L + D1L + D2L,
        D0L + D1L + 0,
        D0L + D1L + D2R,
        D0L + 0 + D2L,
        D0L + 0 + 0,
        D0L + 0 + D2R,
        D0L + D1R + D2L,
        D0L + D1R + 0,
        D0L + D1R + D2R,
        0 + D1L + D2L,
        0 + D1L + 0,
        0 + D1L + D2R,
        0 + 0 + D2L,
        0 + 0 + D2R,
        0 + D1R + D2L,
        0 + D1R + 0,
        0 + D1R + D2R,
        D0R + D1L + D2L,
        D0R + D1L + 0,
        D0R + D1L + D2R,
        D0R + 0 + D2L,
        D0R + 0 + 0,
        D0R + 0 + D2R,
        D0R + D1R + D2L,
        D0R + D1R + 0,
        D0R + D1R + D2R,
    ];

    let mut touched = FxHashSet::<Offset>::with_capacity_and_hasher(MAX_ACTIVE, Default::default());
    let mut counts = [[0u8; BOARD_SIZE]; N_ITER];
    let mut board = [[0u8; BOARD_SIZE]; N_ITER + 1];
    let mut active: ArrayVec<[ArrayVec<[Offset; MAX_ACTIVE]>; N_ITER + 1]> = Default::default();

    for _ in 0..N_ITER + 1 {
        active.push(Default::default());
    }

    const CENTER: Offset = ((D0 >> 1) * (D1 * D2)) as Offset;

    for &(x, y) in &parse_board(s) {
        let x = ((x - 4) as Offset) + ((D1 as Offset) >> 1);
        let y = ((y - 4) as Offset) + ((D2 as Offset) >> 1);
        let id = CENTER + y * (D1 as Offset) + x;
        active[0].push(id);
        board[0][id as usize] = 1;
    }

    for step in 0..N_ITER {
        touched.clear();

        let counts = &mut counts[step];
        for id in &active[step] {
            for &offset in &OFFSETS {
                let neighbor = id + offset;
                touched.insert(neighbor);
                *counts.get_mut_at(neighbor as _) += 1;
            }
        }

        let next_active = unsafe { active.as_mut_ptr().add(step + 1) };
        let next_board = unsafe { board.as_mut_ptr().add(step + 1) };

        let board = &board[step];
        for &id in &touched {
            let count = counts.get_at(id as _);
            if count == 3 || (count == 2 && board.get_at(id as _) != 0) {
                unsafe {
                    (*next_active).push_unchecked(id);
                    (*next_board).set_at(id as _, 1);
                }
            }
        }
    }

    active[N_ITER].len() as _
}

#[inline]
pub fn part2(s: &[u8]) -> u64 {
    // two coordinates fit within 0-16, the other two fit within 0-32 => 18 bits
    const D0: usize = 1 << 4;
    const D1: usize = 1 << 4;
    const D2: usize = 1 << 5;
    const D3: usize = 1 << 5;

    // total maximum grid size
    const BOARD_SIZE: usize = D0 * D1 * D2 * D3;

    // max number of active cells
    const MAX_ACTIVE: usize = 1 << 14;

    // absolute offsets in each direction
    type Offset = i32;
    const D0L: Offset = -1 * (D1 as Offset) * (D2 as Offset) * (D3 as Offset);
    const D0R: Offset = 1 * (D1 as Offset) * (D2 as Offset) * (D3 as Offset);
    const D1L: Offset = -1 * (D2 as Offset) * (D3 as Offset);
    const D1R: Offset = 1 * (D2 as Offset) * (D3 as Offset);
    const D2L: Offset = -1 * (D3 as Offset);
    const D2R: Offset = 1 * (D3 as Offset);
    const D3L: Offset = -1;
    const D3R: Offset = 1;

    // total number of neighbors
    const N_NEIGHBORS: usize = 80;

    // each of the 80 neighbors in absolute terms
    const OFFSETS: [Offset; N_NEIGHBORS] = [
        D0L + D1L + D2L + D3L,
        D0L + D1L + D2L + 0,
        D0L + D1L + D2L + D3R,
        D0L + D1L + 0 + D3L,
        D0L + D1L + 0 + 0,
        D0L + D1L + 0 + D3R,
        D0L + D1L + D2R + D3L,
        D0L + D1L + D2R + 0,
        D0L + D1L + D2R + D3R,
        D0L + 0 + D2L + D3L,
        D0L + 0 + D2L + 0,
        D0L + 0 + D2L + D3R,
        D0L + 0 + 0 + D3L,
        D0L + 0 + 0 + 0,
        D0L + 0 + 0 + D3R,
        D0L + 0 + D2R + D3L,
        D0L + 0 + D2R + 0,
        D0L + 0 + D2R + D3R,
        D0L + D1R + D2L + D3L,
        D0L + D1R + D2L + 0,
        D0L + D1R + D2L + D3R,
        D0L + D1R + 0 + D3L,
        D0L + D1R + 0 + 0,
        D0L + D1R + 0 + D3R,
        D0L + D1R + D2R + D3L,
        D0L + D1R + D2R + 0,
        D0L + D1R + D2R + D3R,
        0 + D1L + D2L + D3L,
        0 + D1L + D2L + 0,
        0 + D1L + D2L + D3R,
        0 + D1L + 0 + D3L,
        0 + D1L + 0 + 0,
        0 + D1L + 0 + D3R,
        0 + D1L + D2R + D3L,
        0 + D1L + D2R + 0,
        0 + D1L + D2R + D3R,
        0 + 0 + D2L + D3L,
        0 + 0 + D2L + 0,
        0 + 0 + D2L + D3R,
        0 + 0 + 0 + D3L,
        0 + 0 + 0 + D3R,
        0 + 0 + D2R + D3L,
        0 + 0 + D2R + 0,
        0 + 0 + D2R + D3R,
        0 + D1R + D2L + D3L,
        0 + D1R + D2L + 0,
        0 + D1R + D2L + D3R,
        0 + D1R + 0 + D3L,
        0 + D1R + 0 + 0,
        0 + D1R + 0 + D3R,
        0 + D1R + D2R + D3L,
        0 + D1R + D2R + 0,
        0 + D1R + D2R + D3R,
        D0R + D1L + D2L + D3L,
        D0R + D1L + D2L + 0,
        D0R + D1L + D2L + D3R,
        D0R + D1L + 0 + D3L,
        D0R + D1L + 0 + 0,
        D0R + D1L + 0 + D3R,
        D0R + D1L + D2R + D3L,
        D0R + D1L + D2R + 0,
        D0R + D1L + D2R + D3R,
        D0R + 0 + D2L + D3L,
        D0R + 0 + D2L + 0,
        D0R + 0 + D2L + D3R,
        D0R + 0 + 0 + D3L,
        D0R + 0 + 0 + 0,
        D0R + 0 + 0 + D3R,
        D0R + 0 + D2R + D3L,
        D0R + 0 + D2R + 0,
        D0R + 0 + D2R + D3R,
        D0R + D1R + D2L + D3L,
        D0R + D1R + D2L + 0,
        D0R + D1R + D2L + D3R,
        D0R + D1R + 0 + D3L,
        D0R + D1R + 0 + 0,
        D0R + D1R + 0 + D3R,
        D0R + D1R + D2R + D3L,
        D0R + D1R + D2R + 0,
        D0R + D1R + D2R + D3R,
    ];

    let mut touched = FxHashSet::<Offset>::with_capacity_and_hasher(MAX_ACTIVE, Default::default());
    let mut counts = [[0u8; BOARD_SIZE]; N_ITER];
    let mut board = [[0u8; BOARD_SIZE]; N_ITER + 1];
    let mut active: ArrayVec<[ArrayVec<[Offset; MAX_ACTIVE]>; N_ITER + 1]> = Default::default();

    for _ in 0..N_ITER + 1 {
        active.push(Default::default());
    }

    const CENTER: Offset = ((D0 >> 1) * (D1 * D2 * D3) + (D1 >> 1) * (D2 * D3)) as Offset;

    for &(x, y) in &parse_board(s) {
        let x = (x - 4) + ((D3 as Offset) >> 1);
        let y = (y - 4) + ((D2 as Offset) >> 1);
        let id = CENTER + y * (D2 as Offset) + x;
        active[0].push(id);
        board[0][id as usize] = 1;
    }

    for step in 0..N_ITER {
        touched.clear();

        let counts = &mut counts[step];
        for id in &active[step] {
            for &offset in &OFFSETS {
                let neighbor = id + offset;
                touched.insert(neighbor);
                *counts.get_mut_at(neighbor as _) += 1;
            }
        }

        let next_active = unsafe { active.as_mut_ptr().add(step + 1) };
        let next_board = unsafe { board.as_mut_ptr().add(step + 1) };

        let board = &board[step];
        for &id in &touched {
            let count = counts.get_at(id as _);
            if count == 3 || (count == 2 && board.get_at(id as _) != 0) {
                unsafe {
                    (*next_active).push_unchecked(id);
                    (*next_board).set_at(id as _, 1);
                }
            }
        }
    }

    active[N_ITER].len() as _
}

#[test]
fn test_day16_part1() {
    assert_eq!(part1(input()), 240);
}

#[test]
fn test_day16_part2() {
    assert_eq!(part2(input()), 1180);
}
