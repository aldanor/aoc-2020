use crate::utils::ByteSliceExt;

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

#[inline]
fn parse_i16(s: &mut &[u8]) -> i16 {
    let neg = s.get_first() == b'-';
    *s = s.advance(1);
    let mut d = s.get_digit() as i16;
    *s = s.advance(1);
    let c1 = s.get_first().wrapping_sub(b'\n');
    if c1 != 0 {
        d = d * 10 + (c1.wrapping_sub(b'0' - b'\n') as i16);
        *s = s.advance(1);
        let c2 = s.get_first().wrapping_sub(b'\n');
        if c2 != 0 {
            d = d * 10 + (c2.wrapping_sub(b'0' - b'\n') as i16);
            *s = s.advance(1);
        }
    }
    if neg {
        -d
    } else {
        d
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
            let arg = parse_i16(s);
            *s = s.advance(1);
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

    pub fn execute(&mut self) -> i16 {
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

    pub fn find_bug(&self) -> i16 {
        let mut state = [0; MAX_CMDS];
        self.traverse_and_flip(0, 0, NULL, &mut state)
    }

    fn traverse_and_flip(
        &self,
        pos: Pos,
        acc: i16,
        flipped: i16,
        state: &mut [i16; MAX_CMDS],
    ) -> i16 {
        const PENDING: i16 = i16::MAX;
        let i = pos as usize;
        if pos < 0 || pos > (self.n_cmds as Pos) {
            return NULL;
        } else if pos == (self.n_cmds as Pos) {
            return acc;
        } else if state[i] == NULL || state[i] == PENDING {
            return NULL;
        }
        let cmd = unsafe { *self.cmds.get_unchecked(pos as usize) };
        {
            state[i] = PENDING;
            let (mut pos, mut acc) = (pos, acc);
            cmd.execute(&mut pos, &mut acc);
            let acc = self.traverse_and_flip(pos, acc, flipped, state);
            if acc != NULL {
                return acc;
            } else {
                state[i] = NULL;
            }
        }
        if flipped == NULL {
            let (mut pos, mut acc) = (pos, acc);
            cmd.invert().execute(&mut pos, &mut acc);
            let acc = self.traverse_and_flip(pos, acc, 1, state);
            if acc != NULL {
                return acc;
            }
        }
        NULL
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    static INPUT: &[u8] = include_bytes!("input.txt");
    INPUT
}

#[inline]
pub fn part1(s: &[u8]) -> i16 {
    Runner::from_input(s).execute()
}

#[inline]
pub fn part2(s: &[u8]) -> i16 {
    Runner::from_input(s).find_bug()
}
