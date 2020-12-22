use std::collections::VecDeque;

use bigint::U512;
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
static RECURSE_MASKS: [U512; 64] = {
    let mut masks = [U512::default(); 64];
    for i in 0..64 {
        masks[i] = (U512::from(1) << (6 * i)) - U512::from(1);
    }
    masks
};

#[inline]
fn get_recurse_mask(len: usize) -> U512 {
    unsafe { *RECURSE_MASKS.get_unchecked(len) }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct FastDeck {
    cards: U512,
    len: usize,
}

impl FastDeck {
    #[inline]
    pub fn new(deck: impl Iterator<Item = Card>) -> Self {
        let mut deck = deck.collect::<Vec<_>>();
        deck.reverse();
        let mut cards = Default::default();
        let mut len = 0;
        for &c in &deck {
            cards = (cards << 6) | U512::from(c);
            len += 1;
        }
        Self { cards, len }
    }

    #[inline]
    pub fn top(&self) -> Card {
        (self.cards.low_u32() % 64) as _
    }

    #[inline]
    pub fn can_recurse(&self) -> bool {
        self.len > (self.top() as usize)
    }

    #[inline]
    pub fn recurse(&self) -> Self {
        let len = self.top() as usize;
        let cards = (self.cards >> 6) & get_recurse_mask(len);
        Self { len, cards }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        (0..self.len).map(move |i| ((self.cards >> i * 6).low_u32() % 64) as _)
    }

    #[inline]
    pub fn score(&self) -> usize {
        self.iter()
            .enumerate()
            .map(|(i, c)| (self.len - i) * (c as usize))
            .sum()
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
    fn winner_loser(&mut self, winner_is_1: bool) -> (&mut FastDeck, &mut FastDeck) {
        let ptr = self.decks.as_mut_ptr();
        let winner = unsafe { &mut *ptr.add(winner_is_1 as _) };
        let loser = unsafe { &mut *ptr.add(!winner_is_1 as _) };
        (winner, loser)
    }

    #[inline]
    pub fn finish_round(&mut self, winner_is_1: bool) -> bool {
        let (winner, loser) = self.winner_loser(winner_is_1);

        let (winner_card, loser_card) = (winner.top(), loser.top());

        let bottom = U512::from(((loser_card as u16) << 6) | (winner_card as u16));
        winner.cards = (winner.cards >> 6) | (bottom << ((winner.len - 1) * 6));
        winner.len += 1;

        loser.cards = loser.cards >> 6;
        loser.len -= 1;

        loser.len == 0
    }

    #[inline]
    pub fn can_recurse(&self) -> bool {
        self.decks[0].can_recurse() && self.decks[1].can_recurse()
    }

    #[inline]
    fn recurse(&self) -> Self {
        Self::new(self.decks[0].recurse(), self.decks[1].recurse())
    }

    #[inline]
    fn play(&mut self) -> bool {
        let mut history = FxHashSet::with_capacity_and_hasher(1 << 10, Default::default());
        loop {
            if !history.insert(self.decks[0].cards ^ self.decks[1].cards) {
                return false;
            }
            let winner_is_1 = if self.can_recurse() {
                self.recurse().play()
            } else {
                self.decks[1].top() > self.decks[0].top()
            };
            if self.finish_round(winner_is_1) {
                return winner_is_1;
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