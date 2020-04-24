use std::fmt;
use std::f64::EPSILON;
use strum::IntoEnumIterator;
use itertools::Itertools;
use rand::thread_rng;
use rand::seq::SliceRandom;
use failure::Error;
use crate::card::*;
use crate::color::*;
use crate::traits::*;
use crate::trump::*;
use crate::errors::TarotErrorKind;

#[derive(Default, Clone, Debug)]
pub struct Deck (pub Vec<Card>);

pub const MAX_CARDS : usize = 78;
const MAX_POINTS : f64 = 91.0;
const MAX_POINTS_WITHOUT_FOOL : f64 = 87.0;

impl<'a> fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut last_color : Option<&Color> = None;
        let (trumps, colors): (Vec<_>, Vec<_>) = self.0.iter().partition(|c| c.is_trump());
        let trumps_value : Vec<usize> = trumps.iter().filter_map(|t| match t {Card::Trump(v) => Some(*v as usize), _ => None } ).collect();
        if !trumps.is_empty() {
            write!(f, "\n\t{} : {}", TRUMP_COLOR, trumps_value.iter().join(" "))?;
        }
        for colored in colors.iter() {
            if let Card::Color(c, cv) = colored {
                if last_color == Some(&c) {
                    write!(f, "{} ", cv)?
                } else {
                    last_color = Some(&c);
                    match last_color {
                        None => {
                            write!(f, "\t{} : {} ", c, cv)?
                        },
                        _ => write!(f, "\n\t{} : {} ", c, cv)?,
                    }
                }
            }
        }
        Ok(())
    }
}

impl Points for Deck {
    fn points(&self) -> f64 {
        // RULE: if a slam is occuring and player has only fool or everyting except fool, fool = 4 points
        if self.0.len() == MAX_CARDS - 1 && !self.has_fool() {
            MAX_POINTS_WITHOUT_FOOL
        } else if self.len() == 1 && self.has_fool() {
            4.0
        } else {
            self.0.iter().map(Points::points).sum()
        }
    }
}

impl Deck {
    pub fn build_deck() -> Deck {
        let mut d : Vec<Card> = TrumpValue::iter().map(Card::Trump).
             chain(Color::iter().cartesian_product(ColorValue::iter()).map(|(c, cv)| Card::Color(c, cv))).
             collect();
        let mut rng = thread_rng();
        d.shuffle(&mut rng);
        Deck(d)
    }
    pub fn trumps(&self) -> Vec<&Card> {
        self.0.iter().filter(|&card| card.is_trump()).collect()
    }
    pub fn has_fool(&self) -> bool {
        self.0.contains(&Card::Trump(TrumpValue::Fool))
    }
    pub fn has_petit(&self) -> bool {
        self.0.contains(&Card::Trump(TrumpValue::Petit))
    }
    pub fn is_chelem(&self) -> bool {
        // RULE: deck is a chelem if all cards are there or fool is missing
        self.points() == MAX_POINTS || self.points() == MAX_POINTS_WITHOUT_FOOL
    }
    pub fn points_for_oudlers(&self) -> Result<f64, Error> {
        match self.count_oudlers() {
            0 => Ok(56.0),
            1 => Ok(51.0),
            2 => Ok(41.0),
            3 => Ok(36.0),
            _ => Err(TarotErrorKind::InvalidCase.into()),
        }
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn petit_sec(&self) -> bool {
        self.0.iter().fold(0, |acc, c| match &c { Card::Trump(c) => {acc + *c as usize}, _ => acc}) == 1
    }
    pub fn discardables(&self, discard: usize) -> Vec<usize> {
        let choices : Vec<usize> = self.0.iter().enumerate().filter(|(_, card)| card.discardable()).map(|(i, _)| i).collect();
        if choices.len() < discard {
            self.0.iter().enumerate().filter(|(_, card)| card.discardable_forced()).map(|(i, _)| i).collect()
        } else {
            choices
        }
    }
    pub fn count_trumps(&self) -> usize {
        self.0.iter().filter(|card| card.is_trump()).count()
    }
    pub fn count_oudlers(&self) -> usize {
        self.0.iter().filter(|card| card.is_oudler()).count()
    }
    pub fn count_tete(&self, cv: ColorValue) -> usize {
        self.0.iter().filter(|card| match card { Card::Color(_, v) => v == &cv, _ => false }).count()
    }
    pub fn misere_tete(&self) -> bool {
        !self.0.iter().any(|card| match card { Card::Color(_, v) => v.points() - 0.5 < EPSILON, _ => false })
    }
    pub fn give(&mut self, size: usize) -> Deck {
        Deck(self.0.drain(0..size).collect())
    }
    pub fn give_all(&mut self) -> Deck {
        Deck(self.0.drain(..).collect())
    }
    pub fn give_low(&mut self) -> Option<Card> {
        let low_index = &self.0.iter().enumerate().filter_map(|(i, c)| if c.points() - 0.5 < EPSILON { Some(i) } else { None }).next();
        if let Some(index) = low_index {
            Some(self.0.remove(index.to_owned()))
        } else {
            None
        }
    }
    pub fn append(&mut self, deck: &mut Deck) {
        self.0.append(&mut deck.0);
    }
    pub fn push(&mut self, card: Card){
        self.0.push(card);
    }
    pub fn sort(&mut self) {
        self.0.sort();
    }
}

#[test]
fn deck_tests() {
    let stack = Deck::build_deck();

    assert!(stack.len() == MAX_CARDS);
    assert!(stack.points() == MAX_POINTS);

    let empty = Deck::default();
    assert!(empty.is_empty());
}

