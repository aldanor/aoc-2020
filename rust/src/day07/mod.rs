use crate::utils::ByteSliceExt;

const N_COLUMNS: usize = 32;
const N_ROWS: usize = 1024;

type Id = u16;
type GraphMatrix = [[Id; N_COLUMNS]; N_ROWS];

#[derive(Debug)]
struct Lookup2 {
    map: [u8; 1 << 16],
    count: u8,
}

impl Lookup2 {
    pub fn new() -> Self {
        Self {
            map: [0xff; 1 << 16],
            count: 0,
        }
    }

    #[inline]
    pub fn get(&mut self, key: u16) -> u8 {
        unsafe {
            let value = self.map.get_unchecked_mut(key as usize);
            if *value == 0xff {
                *value = self.count;
                self.count += 1;
            }
            *value
        }
    }
}

#[derive(Debug)]
pub struct Lookup6 {
    tables: [Lookup2; 3],
    master: [Id; 1 << 15],
    count: Id,
}

impl Lookup6 {
    pub fn new() -> Self {
        Self {
            tables: [Lookup2::new(), Lookup2::new(), Lookup2::new()],
            master: [Id::MAX; 1 << 15],
            count: 0,
        }
    }

    #[inline]
    pub fn get(&mut self, b0: u16, b1: u16, b2: u16) -> Id {
        let i0 = self.tables[0].get(b0) as u16;
        let i1 = self.tables[1].get(b1) as u16;
        let i2 = self.tables[2].get(b2) as u16;
        let i = (i0 << 10) | (i1 << 5) | i2;
        unsafe {
            let v = self.master.get_unchecked_mut(i as usize);
            if *v == Id::MAX {
                *v = self.count;
                self.count += 1;
            }
            *v
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    CountParents,
    CountChildren,
}

#[derive(Debug)]
pub struct Graph {
    graph: GraphMatrix,
    mode: Mode,
    table: Lookup6,
}

impl Graph {
    pub fn new(mode: Mode) -> Self {
        let graph = [[0; N_COLUMNS]; N_ROWS];
        let table = Lookup6::new();
        Self { graph, mode, table }
    }

    #[inline]
    pub fn encode_id(&mut self, adj: &[u8], col: &[u8]) -> Id {
        let b0 = adj.get_u16();
        let b1 = col.get_u16();
        let b2 = col[2..].get_u16();
        self.table.get(b0, b1, b2) as _
    }

    #[inline]
    fn parse_id<'a>(&mut self, s: &'a [u8], i: usize) -> (&'a [u8], Id) {
        let adj = s;
        let col = s.skip_past(b' ', 0);
        let id = self.encode_id(adj, col);
        let s = col.skip_past(b' ', i);
        (s, id)
    }

    #[inline]
    fn parse_line<'a>(&mut self, s: &'a [u8]) -> &'a [u8] {
        let (s, src) = self.parse_id(s, 13);
        if s.check_first(b'n') {
            return s.skip_past(b'.', 1);
        }
        let mut p = s;
        let row = unsafe { self.get_row(src) };
        loop {
            let n = p.get_digit() as u16;
            let (s, dst) = self.parse_id(&p[2..], 3 + ((n != 1) as usize));
            match self.mode {
                Mode::CountParents => unsafe { self.add_node_parents(src, dst) },
                Mode::CountChildren => unsafe { self.add_node_children(row, dst, n) },
            }
            p = &s[2..];
            if s.check_first(b'.') {
                return p;
            }
        }
    }

    #[inline]
    pub fn parse_input(&mut self, mut s: &[u8]) {
        while s.len() > 1 {
            s = self.parse_line(s);
        }
    }

    #[inline]
    unsafe fn get_row(&mut self, row: Id) -> *mut Id {
        (*self.graph.as_mut_ptr())
            .as_mut_ptr()
            .add((row as usize) * N_COLUMNS)
    }

    #[inline]
    unsafe fn add_node_parents(&mut self, src: Id, dst: Id) {
        let row = self.get_row(dst);
        *row += 1;
        *row.add(*row as usize) = src;
    }

    #[inline]
    unsafe fn add_node_children(&mut self, row: *mut Id, dst: Id, n: Id) {
        let offset = 1 + 2 * (*row as usize);
        *row += 1;
        *row.add(offset) = dst;
        *row.add(offset + 1) = n;
    }

    #[inline]
    fn count_parents(&self, target: Id, found: &mut [u8; N_ROWS]) {
        let target = target as usize;
        let row = &self.graph[target];
        let len = row[0] as usize;
        found[target] = 1;
        for i in 0..len {
            self.count_parents(row[i + 1], found);
        }
    }

    #[inline]
    fn count_children(&self, target: Id) -> u32 {
        let target = target as usize;
        let row = &self.graph[target];
        let len = row[0] as usize;
        let mut count = 1;
        for i in 0..len {
            let dst = row[i * 2 + 1];
            let n = row[i * 2 + 2];
            count += (n as u32) * self.count_children(dst);
        }
        count
    }

    #[inline]
    pub fn count(&self, target: Id) -> u32 {
        match self.mode {
            Mode::CountParents => {
                let mut found = [0; N_ROWS];
                self.count_parents(target, &mut found);
                (found.iter().map(|&x| x as u16).sum::<u16>() - 1) as _
            }
            Mode::CountChildren => self.count_children(target) - 1,
        }
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    static INPUT: &[u8] = include_bytes!("input.txt");
    INPUT
}

#[inline]
pub fn part1(s: &[u8]) -> u32 {
    let mut g = Graph::new(Mode::CountParents);
    g.parse_input(s);
    let target = g.encode_id(b"shiny", b"gold");
    g.count(target)
}

#[inline]
pub fn part2(s: &[u8]) -> u32 {
    let mut g = Graph::new(Mode::CountChildren);
    g.parse_input(s);
    let target = g.encode_id(b"shiny", b"gold");
    g.count(target)
}
