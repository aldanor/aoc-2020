use std::ops::Index;

use crate::utils::*;

use arrayvec::ArrayVec;

const MAX_RULES: usize = 256;

type Id = u8;
type Match<'a> = Option<&'a [u8]>;
type CharMap = [u8; 256];

#[derive(Debug, Copy, Clone)]
enum Term {
    AnyChar,
    Pair(u8, u8),
    Prepend(Id, u8),
    Append(Id, u8),
    Concat(Id, Id),
    Char(u8),
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
        match (term1, term2) {
            (Term::Char(a), Term::Char(b)) if a != b => Rule::Just(Term::AnyChar),
            _ => Rule::Either(term1, term2),
        }
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

impl Rules {
    #[inline]
    fn match_char<'a>(&self, s: &'a [u8], c: u8) -> Match<'a> {
        if s.get_first() == c {
            Some(&s[1..])
        } else {
            None
        }
    }

    #[inline]
    fn match_term<'a>(&self, s: &'a [u8], term: &Term, fixed: bool) -> Match<'a> {
        if s.is_empty() {
            return None;
        }
        match *term {
            Term::AnyChar => Some(&s[1..]),
            Term::Pair(c1, c2) => {
                if s.get_at(0) == c1 && s.get_at(1) == c2 {
                    Some(&s[2..])
                } else {
                    None
                }
            }
            Term::Append(id, c) => {
                if !fixed {
                    self.match_rule(s, id, fixed)
                        .and_then(|s| self.match_char(s, c))
                } else {
                    if s.get_last() == c {
                        self.match_rule(&s[..s.len() - 1], id, fixed)
                    } else {
                        None
                    }
                }
            }
            Term::Prepend(id, c) => self
                .match_char(s, c)
                .and_then(|s| self.match_rule(s, id, fixed)),
            Term::Concat(a, b) => self
                .match_rule(s, a, false) // can't allow greedy match here (or we have to use len)
                .and_then(|s| self.match_rule(s, b, fixed)),
            Term::Char(c) => self.match_char(s, c),
            Term::Just(id) => self.match_rule(s, id, fixed),
        }
    }

    #[inline]
    pub fn match_rule<'a>(&self, s: &'a [u8], id: Id, fixed_size: bool) -> Match<'a> {
        match self[id] {
            Rule::Either(ref a, ref b) => self
                .match_term(s, a, fixed_size)
                .or_else(|| self.match_term(s, b, fixed_size)),
            Rule::Just(ref term) => self.match_term(s, term, fixed_size),
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

type PatternLengths = ArrayVec<[u8; 16]>;
type PatternLengthsArray = ArrayVec<[PatternLengths; MAX_RULES]>;

fn compute_pattern_lengths_for_term(
    rules: &Rules,
    term: &Term,
    arr: &mut PatternLengthsArray,
) -> PatternLengths {
    let mut lengths = PatternLengths::new();
    match *term {
        Term::Just(id) => {
            &update_pattern_lengths_for_rule(rules, id, arr);
            for &n in &arr[id as usize] {
                lengths.push(n);
            }
        }
        Term::AnyChar | Term::Char(_) => lengths.push(1),
        Term::Pair(_, _) => lengths.push(2),
        Term::Prepend(id, _) | Term::Append(id, _) => {
            update_pattern_lengths_for_rule(rules, id, arr);
            for &n in &arr[id as usize] {
                lengths.push(n + 1);
            }
        }
        Term::Concat(id1, id2) => {
            update_pattern_lengths_for_rule(rules, id1, arr);
            update_pattern_lengths_for_rule(rules, id2, arr);
            for n1 in &arr[id1 as usize] {
                for n2 in &arr[id2 as usize] {
                    lengths.push(n1 + n2);
                }
            }
        }
    }
    lengths
}

fn update_pattern_lengths_for_rule(rules: &Rules, id: Id, arr: &mut PatternLengthsArray) {
    if !arr[id as usize].is_empty() {
        return;
    }
    arr[id as usize] = match rules[id].clone() {
        Rule::Just(ref term) => compute_pattern_lengths_for_term(rules, term, arr),
        Rule::Either(ref term1, ref term2) => {
            let mut lengths = compute_pattern_lengths_for_term(rules, term1, arr);
            for &n2 in &compute_pattern_lengths_for_term(rules, term2, arr) {
                if !lengths.contains(&n2) {
                    lengths.push(n2);
                }
            }
            lengths
        }
    };
}

#[inline]
fn compute_pattern_lengths(rules: &Rules) -> ArrayVec<[usize; MAX_RULES]> {
    let mut counts = PatternLengthsArray::new();
    for _ in 0..rules.0.len() {
        counts.push(Default::default());
    }
    for i in 0..rules.0.len() {
        update_pattern_lengths_for_rule(rules, i as _, &mut counts);
    }
    // NOTE: if patterns were of varying lengths, we could just disable greedy
    // approach in parts 1/2 by setting fixed_size=false, it would still work
    assert!(counts.iter().all(|c| c.len() == 1));
    counts.into_iter().map(|c| c[0] as usize).collect()
}

#[inline]
pub fn part1(mut s: &[u8]) -> u16 {
    let rules = parse_rules(&mut s);
    let lengths = compute_pattern_lengths(&rules);
    let root_len = lengths[0];
    parse_inputs(s)
        .map(|line| {
            if line.len() != root_len {
                0
            } else {
                rules
                    .match_rule(line, 0, true)
                    .map(|s| s.is_empty() as u16)
                    .unwrap_or(0)
            }
        })
        .sum()
}

#[inline]
pub fn part2(mut s: &[u8]) -> u16 {
    let rules = parse_rules(&mut s);
    let lengths = compute_pattern_lengths(&rules);
    parse_inputs(s)
        .map(|mut line| {
            let (mut n_42, mut n_31) = (0, 0);
            for (n, id) in &mut [(&mut n_42, 42), (&mut n_31, 31)] {
                let len = lengths[*id as usize];
                while !line.is_empty() {
                    let s = rules.match_rule(&line[..len], *id as _, true);
                    if s.map(|s| s.is_empty()).unwrap_or(false) {
                        **n += 1;
                        line = &line[len..];
                    } else {
                        break;
                    }
                }
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
