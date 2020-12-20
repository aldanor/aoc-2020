use std::num::NonZeroU8;
use std::ops::Index;

use crate::utils::*;

use arrayvec::ArrayVec;

const MAX_RULES: usize = 256;

type Id = u8;
type Match = Option<NonZeroU8>;
type CharMap = [u8; 256];

#[derive(Debug, Copy, Clone)]
enum Term {
    Prepend(Id, u8),
    Append(Id, u8),
    Concat(Id, Id),
    Char(u8),
    Pair(u8, u8),
    Just(Id),
}

#[derive(Debug, Copy, Clone)]
enum Rule {
    Either(Term, Term),
    Just(Term),
}

#[derive(Debug, Clone)]
struct Rules(ArrayVec<[Rule; MAX_RULES]>);

impl Index<Id> for Rules {
    type Output = Rule;

    fn index(&self, index: u8) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

#[inline(always)]
fn read_id(s: &mut &[u8], skip: usize) -> Id {
    parse_int_fast_skip_custom(s, 1, 3, skip)
}

#[inline]
fn parse_term(id1: Id, id2: Option<Id>, map: &CharMap) -> Term {
    let c1 = map.get_at(id1 as _);
    if let Some(id2) = id2 {
        let c2 = map.get_at(id2 as _);
        match (c1, c2) {
            (0, 0) => Term::Concat(id1, id2),
            (0, c) => Term::Append(id1, c),
            (c, 0) => Term::Prepend(id2, c),
            (c1, c2) => Term::Pair(c1, c2),
        }
    } else {
        match c1 {
            0 => Term::Just(id1),
            c => Term::Char(c),
        }
    }
}

#[inline]
fn parse_pair(s: &mut &[u8]) -> (Id, Option<Id>) {
    let id1 = read_id(s, 0);
    if s.get_at(1).is_ascii_digit() && s.get_first() != b'\n' {
        *s = s.advance(1);
        (id1, Some(read_id(s, 0)))
    } else {
        (id1, None)
    }
}

#[inline]
fn parse_rule(s: &mut &[u8], map: &CharMap) -> (Id, Rule) {
    let rule_id = read_id(s, 2);

    if s.get_first() == b'"' {
        let c = s.get_at(1);
        *s = s.advance(4);
        return (rule_id, Rule::Just(Term::Char(c)));
    }

    let (id1, id2) = parse_pair(s);
    let term1 = parse_term(id1, id2, map);

    let rule = if s.get_first() == b'\n' {
        *s = s.advance(1);
        Rule::Just(term1)
    } else {
        *s = s.advance(3);
        let (id1, id2) = parse_pair(s);
        let term2 = parse_term(id1, id2, map);
        *s = s.advance(1);
        Rule::Either(term1, term2)
    };

    (rule_id, rule)
}

#[inline]
fn parse_charmap(s: &[u8], map: &mut CharMap) {
    for i in memchr::memchr_iter(b'"', s) {
        let c = s.get_at(i + 1);
        if c == b'\n' {
            continue;
        }
        let mut i = i - 4;
        while s.get_at(i) != b'\n' {
            i -= 1;
        }
        let mut s = &s[i + 1..];
        let rule_id = read_id(&mut s, 0);
        map[rule_id as usize] = c;
    }
}

#[inline]
fn parse_rules(s: &mut &[u8]) -> Rules {
    let mut map = [0; 256];
    parse_charmap(*s, &mut map);

    let mut rules_and_ids = ArrayVec::<[_; MAX_RULES]>::new();
    while s.get_first() != b'\n' {
        unsafe { rules_and_ids.push_unchecked(parse_rule(s, &map)) };
    }
    rules_and_ids.sort_unstable_by_key(|&(id, _)| id);

    let mut rules = ArrayVec::new();
    unsafe { rules.set_len(rules_and_ids.len()) };
    for (i, &(id, rule)) in rules_and_ids.iter().enumerate() {
        debug_assert_eq!(i as Id, id);
        rules[i] = rule;
    }

    Rules(rules)
}

#[inline]
fn match_len(n: impl Into<u8>) -> NonZeroU8 {
    unsafe { NonZeroU8::new_unchecked(n.into()) }
}

impl Rules {
    #[inline]
    fn match_char(&self, s: &[u8], c: u8) -> Match {
        if s.get_first() == c {
            Some(match_len(1))
        } else {
            None
        }
    }

    #[inline]
    fn match_term(&self, s: &[u8], term: &Term) -> Match {
        if s.is_empty() {
            return None;
        }
        match *term {
            Term::Append(id, c) => self.match_rule(s, id).and_then(|n| {
                self.match_char(&s[n.get() as usize..], c)
                    .map(|_| match_len(n.get() + 1))
            }),
            Term::Prepend(id, c) => self
                .match_char(s, c)
                .and_then(|_| self.match_rule(&s[1..], id).map(|n| match_len(n.get() + 1))),
            Term::Concat(a, b) => self.match_rule(s, a).and_then(|n1| {
                self.match_rule(&s[n1.get() as usize..], b)
                    .map(|n2| match_len(n1.get() + n2.get()))
            }),
            Term::Char(c) => self.match_char(s, c),
            Term::Pair(c1, c2) => self
                .match_char(s, c1)
                .and_then(|_| self.match_char(&s[1..], c2).map(|_| match_len(2))),
            Term::Just(id) => self.match_rule(s, id),
        }
    }

    #[inline]
    pub fn match_rule(&self, s: &[u8], id: Id) -> Match {
        match self[id] {
            Rule::Either(ref a, ref b) => self.match_term(s, a).or_else(|| self.match_term(s, b)),
            Rule::Just(ref term) => self.match_term(s, term),
        }
    }
}

#[inline]
fn parse_inputs(mut s: &[u8]) -> impl Iterator<Item = &[u8]> + '_ {
    while s.get_first() == b'\n' {
        s = s.advance(1);
    }
    std::iter::from_fn(move || {
        let i = memchr::memchr(b'\n', s).unwrap_or(0);
        if i != 0 {
            let line = &s[..i];
            s = s.advance(i + 1);
            Some(line)
        } else {
            None
        }
    })
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> u16 {
    let rules = parse_rules(&mut s);
    parse_inputs(s)
        .map(|line| {
            rules
                .match_rule(line, 0)
                .map(|n| n.get() == (line.len() as u8))
                .unwrap_or(false) as u16
        })
        .sum()
}

#[inline]
pub fn part2(mut s: &[u8]) -> u16 {
    let rules = parse_rules(&mut s);
    parse_inputs(s)
        .map(|mut line| {
            let mut n_42 = 0;
            while let Some(n) = rules.match_rule(line, 42) {
                n_42 += 1;
                line = &line[n.get() as usize..];
            }
            let mut n_31 = 0;
            while let Some(n) = rules.match_rule(line, 31) {
                n_31 += 1;
                line = &line[n.get() as usize..];
            }
            (line.is_empty() && n_42 > n_31 && n_31 != 0) as u16
        })
        .sum()
}

#[test]
fn test_day19_part1() {
    assert_eq!(part1(input()), 142);
}

#[test]
fn test_day19_part2() {
    assert_eq!(part2(input()), 294);
}
