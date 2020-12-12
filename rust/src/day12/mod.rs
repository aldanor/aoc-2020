use crate::utils::*;

type Coord = i16;
type Distance = i32;

#[derive(Debug, Copy, Clone, Default)]
pub struct Vec2(Coord, Coord);

impl Vec2 {
    #[inline]
    pub fn shift(&mut self, dir: Self, scale: Coord) {
        *self = Self(self.0 + scale * dir.0, self.1 + scale * dir.1)
    }

    #[inline]
    pub fn turn_right(&mut self) {
        *self = Self(self.1, -self.0);
    }

    #[inline]
    pub fn turn_left(&mut self) {
        *self = Self(-self.1, self.0);
    }

    #[inline]
    pub fn reverse(&mut self) {
        *self = Self(-self.0, -self.1);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    loc: Vec2,
    dir: Vec2,
}

impl Position {
    pub fn new(x: Coord, y: Coord) -> Self {
        let (loc, dir) = (Vec2(0, 0), Vec2(x, y));
        Self { loc, dir }
    }

    #[inline]
    pub fn parse_and_update(&mut self, s: &mut &[u8], to_shift: impl Fn(&mut Self) -> &mut Vec2) {
        let dir = s.get_first();
        *s = s.advance(1);
        let num = parse_int_fast(s, 1, 3);
        match (dir, num) {
            (b'N', num) => to_shift(self).1 += num,
            (b'E', num) => to_shift(self).0 += num,
            (b'S', num) => to_shift(self).1 -= num,
            (b'W', num) => to_shift(self).0 -= num,
            (b'F', num) => self.loc.shift(self.dir, num),
            (b'R', 90) | (b'L', 270) => self.dir.turn_right(),
            (b'R', 270) | (b'L', 90) => self.dir.turn_left(),
            _ => self.dir.reverse(),
        }
    }

    pub fn manhattan_distance(self) -> Distance {
        Distance::from(self.loc.0.abs()) + Distance::from(self.loc.1.abs())
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> Distance {
    let mut pos = Position::new(1, 0);
    while s.len() > 1 {
        pos.parse_and_update(&mut s, |p| &mut p.loc);
    }
    pos.manhattan_distance()
}

#[inline]
pub fn part2(mut s: &[u8]) -> Distance {
    let mut pos = Position::new(10, 1);
    while s.len() > 1 {
        pos.parse_and_update(&mut s, |p| &mut p.dir);
    }
    pos.manhattan_distance()
}

#[test]
fn test_day12_part1() {
    assert_eq!(part1(input()), 759);
}

#[test]
fn test_day12_part2() {
    assert_eq!(part2(input()), 45763);
}
