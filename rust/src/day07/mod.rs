use std::hint::unreachable_unchecked;

trait UnwrapUnchecked<T> {
    fn unwrap_unchecked(self) -> T;
}

impl<T> UnwrapUnchecked<T> for Option<T> {
    #[inline]
    fn unwrap_unchecked(self) -> T {
        self.unwrap_or_else(|| unsafe { unreachable_unchecked() })
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    static INPUT: &[u8] = include_bytes!("input.txt");
    INPUT
}

const C: usize = 32;
const R: usize = 1024;
type Id = u16;
type G = [[Id; C]; R];

#[derive(Debug)]
struct Table2 {
    map: [u8; 1 << 16],
    count: u8,
}

impl Table2 {
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
pub struct Table6 {
    tables: [Table2; 3],
    master: [Id; 1 << 15],
    count: Id,
}

impl Table6 {
    pub fn new() -> Self {
        Self {
            tables: [Table2::new(), Table2::new(), Table2::new()],
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
    graph: G,
    mode: Mode,
    table: Table6,
}

impl Graph {
    pub fn new(mode: Mode) -> Self {
        let graph = [[0; C]; R];
        let table = Table6::new();
        Self { graph, mode, table }
    }

    #[inline]
    pub fn encode_id_str(&mut self, adj: &str, col: &str) -> Id {
        assert!(adj.len() >= 4 && col.len() >= 4);
        unsafe { self.encode_id_ptr(adj.as_ptr(), col.as_ptr()) }
    }

    #[inline]
    pub unsafe fn encode_id_ptr(&mut self, adj: *const u8, col: *const u8) -> Id {
        let b0 = u16::from_ne_bytes(*adj.cast());
        let b1 = u16::from_ne_bytes(*col.cast());
        let b2 = u16::from_ne_bytes(*col.add(2).cast());
        self.table.get(b0, b1, b2) as _
    }

    #[inline]
    pub fn parse_input(&mut self, s: &[u8]) {
        unsafe {
            let mut ptr = s.as_ptr();
            let end = ptr.add(s.len() - 1);
            while ptr < end {
                ptr = self.parse_line(ptr);
            }
        };
    }

    #[inline]
    unsafe fn parse_line(&mut self, mut p: *const u8) -> *const u8 {
        let skip_past = |p: *const u8, c: u8| {
            p.add(memchr::memchr(c, std::slice::from_raw_parts(p, 256)).unwrap_unchecked() + 1)
        };
        let p_adj = p;
        p = skip_past(p, b' ');
        let src = self.encode_id_ptr(p_adj, p);
        p = skip_past(p, b' ').add(13);
        if *p == b'n' {
            return skip_past(p, b'.').add(1);
        }
        loop {
            let n = (*p - b'0') as u16;
            p = p.add(2);
            let p_adj = p;
            p = skip_past(p, b' ');
            let dst = self.encode_id_ptr(p_adj, p);
            self.add_node(src, dst, n);
            p = skip_past(p, b' ').add(3 + ((n != 1) as usize));
            if *p == b'.' {
                return p.add(2);
            }
            p = p.add(2);
        }
    }

    #[inline]
    unsafe fn get_row(&mut self, row: Id) -> *mut Id {
        (*self.graph.as_mut_ptr())
            .as_mut_ptr()
            .add((row as usize) * C)
    }

    #[inline]
    unsafe fn add_node(&mut self, src: Id, dst: Id, n: u16) {
        match self.mode {
            Mode::CountParents => {
                let row = self.get_row(dst);
                *row += 1;
                *row.add(*row as usize) = src;
            }
            Mode::CountChildren => {
                let row = self.get_row(src);
                let offset = 1 + 2 * (*row as usize);
                *row += 1;
                *row.add(offset) = dst;
                *row.add(offset + 1) = n;
            }
        }
    }

    #[inline]
    fn count_parents(&self, target: Id, found: &mut [u8; R]) {
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
                let mut found = [0; R];
                self.count_parents(target, &mut found);
                (found.iter().map(|&x| x as u16).sum::<u16>() - 1) as _
            }
            Mode::CountChildren => self.count_children(target) - 1,
        }
    }
}

pub fn part1(s: &[u8]) -> u32 {
    let mut g = Graph::new(Mode::CountParents);
    g.parse_input(s);
    let target = g.encode_id_str("shiny", "gold");
    g.count(target)
}

pub fn part2(s: &[u8]) -> u32 {
    let mut g = Graph::new(Mode::CountChildren);
    g.parse_input(s);
    let target = g.encode_id_str("shiny", "gold");
    g.count(target)
}
