use std::collections::VecDeque;
use std::slice;

use packed_simd_2::u8x64;
use rustc_hash::FxHashSet;

use crate::utils::*;

type Card = u8;
type Deck = VecDeque<Card>;

#[inline]
fn parse_deck(s: &mut &[u8]) -> Deck {
    let mut deck = VecDeque::with_capacity(64);
    *s = s.advance(10);
    while s.get_first() != b'\n' && !s.is_empty() {
        deck.push_back(parse_int_fast(s, 1, 2));
    }
    *s = s.advance(1);
    deck
}

#[inline]
fn parse_decks(mut s: &[u8]) -> (Deck, Deck) {
    let deck1 = parse_deck(&mut s);
    let deck2 = parse_deck(&mut s);
    (deck1, deck2)
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> usize {
    let (mut deck1, mut deck2) = parse_decks(s);
    while !deck1.is_empty() && !deck2.is_empty() {
        for _ in 0..deck1.len().min(deck2.len()) {
            let c1 = deck1.pop_front().unwrap();
            let c2 = deck2.pop_front().unwrap();
            if c1 > c2 {
                deck1.push_back(c1);
                deck1.push_back(c2);
            } else {
                deck2.push_back(c2);
                deck2.push_back(c1);
            }
        }
    }
    let winner = if deck1.is_empty() { &deck2 } else { &deck1 };
    winner
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &c)| (i + 1) * (c as usize))
        .sum()
}

#[ctor::ctor]
static TRUNCATE_MASKS: [Deck512; 64] = {
    let mut masks = [Deck512::default(); 64];
    for i in 0..64 {
        for j in 0..i {
            masks[i].as_bytes_mut()[j] = 0xff;
        }
    }
    masks
};

#[inline]
fn hash_combine(a: u64, b: u64) -> u64 {
    a ^ (b.wrapping_add(a << 6).wrapping_add(b >> 2))
}

// Raw container for storing decks; not explicitly aware of its size
#[repr(align(512))] // align to 512-bit boundary for faster SIMD
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct Deck512([u64; 8]); // 8th byte is ignored

impl Deck512 {
    #[inline]
    pub fn top(&self) -> Card {
        (self.0[0] % 256) as _
    }

    #[inline]
    pub fn pop(&mut self) {
        // remove top card
        unsafe {
            self.write_u8x64(shuffle!(
                self.read_u8x64(),
                [
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                    23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42,
                    43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
                    63, 0
                ]
            ));
            self.as_bytes_mut()[63] = 0;
        }
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        // leave only `len` cards
        let mask = TRUNCATE_MASKS[len];
        unsafe { self.write_u8x64(self.read_u8x64() & mask.read_u8x64()) };
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        let mut hash = 0;
        for i in 0..7 {
            // don't need the last number as it's always zero
            hash = hash_combine(hash, self.0[i]);
        }
        hash
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // return all cards (includes trailing zeros)
        unsafe { slice::from_raw_parts(self.0.as_ptr() as *const _, 64) }
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        // returns all cards (includes trailing zeros)
        unsafe { slice::from_raw_parts_mut(self.0.as_mut_ptr() as *mut _, 64) }
    }

    #[inline]
    pub unsafe fn read_u8x64(&self) -> u8x64 {
        u8x64::from_slice_aligned_unchecked(self.as_bytes())
    }

    #[inline]
    pub unsafe fn write_u8x64(&mut self, reg: u8x64) {
        reg.write_to_slice_aligned_unchecked(self.as_bytes_mut())
    }

    #[inline]
    pub fn max(&self) -> Card {
        // find the max card
        unsafe { self.read_u8x64().max_element() }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct FastDeck {
    cards: Deck512,
    len: usize,
}

impl FastDeck {
    #[inline]
    pub fn new(deck: impl Iterator<Item = Card>) -> Self {
        let deck = deck.collect::<Vec<_>>();
        let mut cards = Deck512::default();
        for (i, &c) in deck.iter().enumerate() {
            cards.as_bytes_mut()[i] = c;
        }
        let len = deck.len();
        Self { cards, len }
    }

    #[inline]
    pub fn pop(&mut self) -> Card {
        let top = self.cards.top();
        self.cards.pop();
        self.len -= 1;
        top
    }

    #[inline]
    pub fn can_recurse(&self, top: Card) -> bool {
        (self.len as u8) >= top
    }

    #[inline]
    pub fn recurse(&self, top: Card) -> Self {
        let mut deck = self.clone();
        deck.len = top as _;
        deck.cards.truncate(deck.len);
        deck
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        self.cards.as_bytes().iter().copied().take(self.len as _)
    }

    #[inline]
    pub fn score(&self) -> usize {
        self.iter()
            .enumerate()
            .map(|(i, c)| (self.len - i) * (c as usize))
            .sum()
    }

    #[inline]
    pub fn max(&self) -> Card {
        self.cards.max()
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.cards.hash()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn push(&mut self, card: Card) {
        self.cards.as_bytes_mut()[self.len] = card;
        self.len += 1;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct FastGame {
    decks: [FastDeck; 2],
}

impl FastGame {
    #[inline]
    pub fn new(deck1: FastDeck, deck2: FastDeck) -> Self {
        let decks = [deck1, deck2];
        Self { decks }
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        hash_combine(self.decks[0].hash(), self.decks[1].hash())
    }

    #[inline]
    pub fn finish_round(&mut self, winner_is_1: bool, c0: Card, c1: Card) -> bool {
        let winner = &mut self.decks[winner_is_1 as usize];
        let cards = [c0, c1];
        winner.push(cards[winner_is_1 as usize]);
        winner.push(cards[!winner_is_1 as usize]);
        let loser = &self.decks[!winner_is_1 as usize];
        loser.is_empty()
    }

    #[inline]
    pub fn pop(&mut self) -> (Card, Card) {
        (self.decks[0].pop(), self.decks[1].pop())
    }

    #[inline]
    pub fn can_recurse(&self, c0: Card, c1: Card) -> bool {
        self.decks[0].can_recurse(c0) && self.decks[1].can_recurse(c1)
    }

    #[inline]
    fn recurse(&self, c0: Card, c1: Card) -> Self {
        Self::new(self.decks[0].recurse(c0), self.decks[1].recurse(c1))
    }

    #[inline]
    fn play(&mut self) -> bool {
        self.play_internal(false)
    }

    #[inline]
    fn play_internal(&mut self, short_circuit: bool) -> bool {
        if self.decks[0].max() > self.decks[1].max() && short_circuit {
            return false; // player 0 has the highest card so he inevitably wins
        }
        let mut history = FxHashSet::with_capacity_and_hasher(1 << 9, Default::default());
        loop {
            let hash = self.hash();
            let (c0, c1) = self.pop();
            let winner_is_1 = if self.can_recurse(c0, c1) {
                self.recurse(c0, c1).play_internal(true)
            } else {
                c1 > c0
            };
            if self.finish_round(winner_is_1, c0, c1) {
                return winner_is_1;
            }
            if !history.insert(hash) {
                return false;
            }
        }
    }
}

#[inline]
pub fn part2(s: &[u8]) -> usize {
    let (deck1, deck2) = parse_decks(s);
    let mut game = FastGame::new(
        FastDeck::new(deck1.iter().copied()),
        FastDeck::new(deck2.iter().copied()),
    );
    let winner_is_1 = game.play();
    game.decks[winner_is_1 as usize].score()
}

#[test]
fn test_day22_part1() {
    assert_eq!(part1(input()), 32083);
}

#[test]
fn test_day22_part2() {
    assert_eq!(part2(input()), 35495);
}
