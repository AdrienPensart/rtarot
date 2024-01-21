use colored::{ColoredString, Colorize};
use derive_more::{Deref, Index, IntoIterator};
use derive_new::new;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::BTreeMap;
use std::fmt;
use strum::IntoEnumIterator;

use crate::card::Card;
use crate::constants::{MAX_CARDS, MAX_POINTS, MAX_POINTS_WITHOUT_FOOL};
use crate::errors::TarotErrorKind;
use crate::points::Points;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::{Discardable, Representation};
use crate::trump::Trump;

#[derive(new, Default, PartialEq, Eq, Clone, Debug, IntoIterator, Index, Deref)]
#[deref(forward)]
pub struct Deck(Vec<Card>);

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut last_color: Option<&Suit> = None;
        let (trumps, colors) = self.trumps_and_colors();
        if !trumps.is_empty() {
            write!(f, "\n\t{}", trumps.iter().join(" "))?;
        }
        for colored in &colors {
            if let Card::Normal(n) = colored {
                if last_color == Some(n.suit()) {
                    write!(f, "{n} ")?;
                } else {
                    last_color = Some(n.suit());
                    match last_color {
                        None => write!(f, "\t{n}")?,
                        _ => write!(f, "\n\t{n} ")?,
                    }
                }
            }
        }
        Ok(())
    }
}

impl Points for Deck {
    fn points(&self) -> OrderedFloat<f64> {
        // RULE: if a slam is occuring and player has only fool or everyting except fool, fool = 4 points
        if self.len() == MAX_CARDS - 1 && !self.has_fool() {
            MAX_POINTS_WITHOUT_FOOL
        } else if self.only_fool() {
            OrderedFloat(4.0)
        } else {
            let mut total = OrderedFloat(0.0);
            for card in self.iter() {
                total += card.points();
            }
            total
        }
    }
}

impl Deck {
    pub fn random() -> Self {
        let mut d: Vec<Card> = Trump::iter()
            .map(Card::Trump)
            .chain(
                Suit::iter()
                    .cartesian_product(SuitValue::iter())
                    .map(|(c, cv)| Card::normal(c, cv)),
            )
            .collect();
        let mut rng = thread_rng();
        d.shuffle(&mut rng);
        Self(d)
    }
    #[must_use]
    pub fn trumps_and_colors(&self) -> (Vec<Card>, Vec<Card>) {
        self.iter().partition(|c| c.is_trump())
    }
    #[must_use]
    pub fn trumps(&self) -> Vec<&Card> {
        self.iter().filter(|&card| card.is_trump()).collect()
    }
    #[must_use]
    pub fn only_fool(&self) -> bool {
        self.len() == 1 && self.contains(&Card::Trump(Trump::Fool))
    }
    #[must_use]
    pub fn has(&self, card: &Card) -> bool {
        self.contains(card)
    }
    #[must_use]
    pub fn has_fool(&self) -> bool {
        self.contains(&Card::Trump(Trump::Fool))
    }
    #[must_use]
    pub fn has_petit(&self) -> bool {
        self.contains(&Card::Trump(Trump::Petit))
    }
    #[must_use]
    pub fn is_chelem(&self) -> bool {
        // RULE: deck is a chelem if all cards are there or fool is missing
        self.points() == MAX_POINTS || self.points() == MAX_POINTS_WITHOUT_FOOL
    }
    pub fn points_for_oudlers(&self) -> Result<OrderedFloat<f64>, TarotErrorKind> {
        match self.count_oudlers() {
            0 => Ok(OrderedFloat(56.0)),
            1 => Ok(OrderedFloat(51.0)),
            2 => Ok(OrderedFloat(41.0)),
            3 => Ok(OrderedFloat(36.0)),
            _ => {
                let only_oudlers = Self(self.oudlers());
                Err(TarotErrorKind::InvalidOudlersCount(only_oudlers))
            }
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[must_use]
    pub fn petit_sec(&self) -> bool {
        self.iter().fold(0, |acc, c| match &c {
            Card::Trump(c) => acc + *c as usize,
            Card::Normal(_) => acc,
        }) == 1
    }
    #[must_use]
    pub fn discardables(&self, discard: usize) -> Vec<usize> {
        let choices: Vec<usize> = self
            .iter()
            .enumerate()
            .filter(|(_, card)| card.discardable())
            .map(|(i, _)| i)
            .collect();
        if choices.len() < discard {
            self.iter()
                .enumerate()
                .filter(|(_, card)| card.discardable_forced())
                .map(|(i, _)| i)
                .collect()
        } else {
            choices
        }
    }
    #[must_use]
    fn oudlers(&self) -> Vec<Card> {
        self.iter()
            .filter(|card| card.is_trump())
            .copied()
            .collect()
    }
    #[must_use]
    pub fn count_trumps(&self) -> usize {
        self.iter().filter(|card| card.is_trump()).count()
    }
    #[must_use]
    pub fn count_oudlers(&self) -> usize {
        self.iter().filter(|card| card.is_oudler()).count()
    }
    #[must_use]
    pub fn count_tete(&self, value: SuitValue) -> usize {
        self.iter()
            .filter(|card| match card {
                Card::Normal(n) => n.value() == &value,
                Card::Trump(_) => false,
            })
            .count()
    }
    #[must_use]
    pub fn misere_tete(&self) -> bool {
        !self.iter().any(|card| match card {
            Card::Normal(n) => n.points() == 0.5,
            Card::Trump(_) => false,
        })
    }
    #[must_use]
    pub fn give(&mut self, size: usize) -> Self {
        Self(self.0.drain(0..size).collect())
    }
    #[must_use]
    pub fn give_all(&mut self) -> Self {
        Self(self.0.drain(..).collect())
    }
    pub fn give_low(&mut self) -> Option<Card> {
        self.iter()
            .enumerate()
            .find_map(|(i, c)| if c.points() == 0.5 { Some(i) } else { None })
            .as_ref()
            .map(|index| self.remove(index.to_owned()))
    }
    pub fn remove(&mut self, index: usize) -> Card {
        self.0.remove(index)
    }
    pub fn extend(&mut self, deck: &Self) {
        self.0.extend(deck.0.clone());
    }
    pub fn push(&mut self, card: Card) {
        self.0.push(card);
    }
    pub fn sort(&mut self) {
        self.0.sort();
    }
    #[must_use]
    pub fn full_repr(&self) -> ColoredString {
        let mut buffers: BTreeMap<usize, String> = BTreeMap::new();
        for card in self.iter() {
            for (index, line) in card.full_repr().lines().enumerate() {
                let line_and_reset = format!("{}\x1B[0m", &line.color(card.color()));
                buffers
                    .entry(index)
                    .and_modify(|buffer| {
                        buffer.push_str(&line_and_reset);
                    })
                    .or_insert(line_and_reset);
            }
        }
        ColoredString::from(buffers.values().join("\n").as_str())
    }
}

#[test]
fn deck_tests() {
    let deck = Deck::random();
    assert_eq!(deck.len(), MAX_CARDS);
    assert_eq!(deck.points(), MAX_POINTS);

    let empty = Deck::default();
    assert!(empty.is_empty());

    let two_cards: Vec<Card> = vec![
        Card::Trump(Trump::_2),
        Card::normal(Suit::Heart, SuitValue::Jack),
        Card::normal(Suit::Spade, SuitValue::Knight),
        Card::normal(Suit::Diamond, SuitValue::Queen),
        Card::normal(Suit::Club, SuitValue::King),
        Card::Trump(Trump::Fool),
    ];
    let test_stack = Deck(two_cards);
    println!("{}", test_stack.full_repr());
}
