use std::iter;
use std::str::from_utf8;

use arrayvec::ArrayVec;
use bigint::U256;
use itertools::Itertools;
use rustc_hash::FxHashMap;

use crate::utils::*;

#[derive(Debug, Copy, Clone, Default)]
struct Food {
    ingredients: U256,
    allergens: u8,
}

#[inline]
fn iter_ingredients(mask: U256) -> impl Iterator<Item = usize> {
    let mut x = mask;
    let one = U256::one();
    iter::from_fn(move || {
        if x.is_zero() {
            None
        } else {
            let n = x.trailing_zeros() as usize;
            x = x & !(one << n);
            Some(n)
        }
    })
}

#[inline]
fn count_ones(mask: U256) -> usize {
    (0..32).map(|i| mask.byte(i).count_ones() as usize).sum()
}

#[derive(Debug, Clone, Default)]
struct Problem<'a> {
    ingredients: ArrayVec<[&'a [u8]; 256]>,
    allergens: ArrayVec<[&'a [u8]; 8]>,
    foods: ArrayVec<[Food; 64]>,
}

#[inline]
fn word_to_key(word: &[u8]) -> u64 {
    let mut key = 0;
    for &c in word {
        key = (key << 8) | (c as u64);
    }
    key
}

impl<'a> Problem<'a> {
    pub fn parse(mut s: &'a [u8]) -> Self {
        let mut ingredient_map = FxHashMap::with_capacity_and_hasher(256, Default::default());
        let mut allergen_map = FxHashMap::with_capacity_and_hasher(8, Default::default());

        let mut problem = Self::default();
        let one = U256::one();

        while s.len() > 1 {
            let mut food = Food::default();

            while s.get_first() != b'(' {
                let mut pos = 2;
                while s.get_at(pos) != b' ' {
                    pos += 1;
                }
                let ingredient = &s[..pos];
                let n = ingredient_map.len();
                food.ingredients = food.ingredients
                    | *ingredient_map
                        .entry(word_to_key(ingredient))
                        .or_insert_with(|| {
                            problem.ingredients.push(ingredient);
                            one << n
                        });
                s = s.advance(pos + 1);
            }

            s = s.advance(10);
            while s.get_first() != b'\n' {
                let pos = s.memchr2(b',', b')');
                let allergen = &s[..pos];
                let n = allergen_map.len();
                food.allergens = food.allergens
                    | *allergen_map
                        .entry(word_to_key(allergen))
                        .or_insert_with(|| {
                            problem.allergens.push(allergen);
                            1 << n
                        });
                s = s.advance(pos + 1 + (s.get_at(pos) == b',') as usize);
            }
            s = s.advance(1);

            problem.foods.push(food);
        }

        problem
    }

    pub fn find_overlaps(&self) -> ArrayVec<[U256; 8]> {
        let mut out = ArrayVec::new();
        for allergen in 0..self.allergens.len() {
            let mut overlap = !U256::zero();
            for food in &self.foods {
                if food.allergens & (1u8 << allergen) != 0 {
                    overlap = overlap & food.ingredients;
                }
            }
            out.push(overlap);
        }
        out
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u16 {
    let problem = Problem::parse(s);
    let overlaps = problem.find_overlaps();
    let no_allergens = !overlaps.iter().fold(U256::zero(), |acc, &x| acc | x);
    problem
        .foods
        .iter()
        .map(|food| count_ones(food.ingredients & no_allergens) as u16)
        .sum()
}

#[inline]
pub fn part2(s: &[u8]) -> String {
    let problem = Problem::parse(s);

    let mut exclude = !U256::zero();
    let mut ingredients = Vec::new();

    let mut overlaps: Vec<_> = problem
        .find_overlaps()
        .iter()
        .enumerate()
        .map(|(i, &o)| (problem.allergens[i], o))
        .collect();
    overlaps.sort_unstable_by_key(|&(_, o)| 256 - count_ones(o));

    while let Some((allergen, mask)) = overlaps.pop() {
        debug_assert_eq!(count_ones(mask), 1);
        ingredients.push((
            allergen,
            problem.ingredients[iter_ingredients(mask).next().unwrap()],
        ));
        exclude = exclude & !mask;
        for entry in &mut overlaps {
            *entry = (entry.0, entry.1 & exclude);
        }
        overlaps.sort_unstable_by_key(|&(_, o)| 256 - count_ones(o));
    }

    ingredients.sort_unstable();
    ingredients
        .iter()
        .map(|&(_, ingredient)| from_utf8(ingredient).unwrap())
        .join(",")
}

#[test]
fn test_day21_part1() {
    assert_eq!(part1(input()), 1679);
}

#[test]
fn test_day21_part2() {
    assert_eq!(
        part2(input()),
        "lmxt,rggkbpj,mxf,gpxmf,nmtzlj,dlkxsxg,fvqg,dxzq"
    );
}
