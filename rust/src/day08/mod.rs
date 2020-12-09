use crate::utils::*;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Op {
    Nop = 0,
    Jmp = 1,
    Acc = 2,
}

impl Default for Op {
    fn default() -> Self {
        Self::Nop
    }
}

impl Op {
    #[inline]
    pub fn invert(self) -> Self {
        match self {
            Self::Jmp => Self::Nop,
            Self::Acc => Self::Acc,
            Self::Nop => Self::Jmp,
        }
    }
}

type Pos = i16;

#[derive(Debug, Copy, Clone, Default)]
pub struct Cmd {
    op: Op,
    arg: i16,
}

impl Cmd {
    #[inline]
    pub fn execute(self, pos: &mut Pos, acc: &mut i16) {
        match self.op {
            Op::Jmp => *pos += self.arg,
            Op::Acc => {
                *pos += 1;
                *acc += self.arg
            }
            Op::Nop => *pos += 1,
        }
    }

    #[inline]
    pub fn invert(self) -> Self {
        Self {
            op: self.op.invert(),
            arg: self.arg,
        }
    }
}

impl Cmd {
    #[inline]
    pub fn try_parse(s: &mut &[u8]) -> Option<Self> {
        if s.len() > 1 {
            let op = match s.get_first() {
                b'j' => Op::Jmp,
                b'a' => Op::Acc,
                _ => Op::Nop,
            };
            *s = s.advance(4);
            let neg = s.get_first() == b'-';
            *s = s.advance(1);
            let mut arg = parse_int_fast::<i16>(s, 1, 3);
            if neg {
                arg = -arg;
            }
            Some(Cmd { op, arg })
        } else {
            None
        }
    }
}

pub const MAX_CMDS: usize = 1024;

const NULL: i16 = i16::MIN;

#[derive(Debug, Copy, Clone)]
pub struct Runner {
    cmds: [Cmd; MAX_CMDS],
    n_cmds: usize,
}

impl Runner {
    pub fn from_input(mut s: &[u8]) -> Self {
        let mut cmds = [Cmd::default(); MAX_CMDS];
        let mut n_cmds = 0;
        while let Some(cmd) = Cmd::try_parse(&mut s) {
            cmds[n_cmds] = cmd;
            n_cmds += 1;
        }
        Self { cmds, n_cmds }
    }

    pub fn execute(&self) -> i16 {
        let (mut pos, mut acc) = (0, 0);
        let mut visited = [false; MAX_CMDS];
        loop {
            let v = unsafe { visited.get_unchecked_mut(pos as usize) };
            if *v {
                break acc;
            }
            unsafe { *self.cmds.get_unchecked(pos as usize) }.execute(&mut pos, &mut acc);
            *v = true;
        }
    }

    #[inline]
    pub fn find_bug_and_execute(&self) -> i16 {
        let mut seen = [false; MAX_CMDS];
        self.traverse_and_flip(0, 0, false, &mut seen)
    }

    #[inline]
    fn traverse_and_flip(
        &self,
        pos: Pos,
        acc: i16,
        flipped: bool,
        seen: &mut [bool; MAX_CMDS],
    ) -> i16 {
        let i = pos as usize;
        if pos < 0 || unsafe { *seen.get_unchecked(i) } {
            return NULL;
        } else if pos >= (self.n_cmds as Pos) {
            return acc;
        }
        unsafe { *seen.get_unchecked_mut(i as usize) = true };
        let cmd = unsafe { *self.cmds.get_unchecked(pos as usize) };
        {
            let (mut pos, mut acc) = (pos, acc);
            cmd.execute(&mut pos, &mut acc);
            let acc = self.traverse_and_flip(pos, acc, flipped, seen);
            if acc != NULL {
                return acc;
            }
        }
        if !flipped {
            let (mut pos, mut acc) = (pos, acc);
            cmd.invert().execute(&mut pos, &mut acc);
            let acc = self.traverse_and_flip(pos, acc, true, seen);
            if acc != NULL {
                return acc;
            }
        }
        NULL
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> i16 {
    Runner::from_input(s).execute()
}

#[inline]
pub fn part2(s: &[u8]) -> i16 {
    Runner::from_input(s).find_bug_and_execute()
}

#[test]
fn test_day08_part1() {
    assert_eq!(part1(input()), 2058);
}

#[test]
fn test_day08_part2() {
    assert_eq!(part2(input()), 1000);
}
