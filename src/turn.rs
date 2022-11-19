use crate::card::Card;
use crate::deck::Deck;
use crate::traits::{Representation, Symbol};
use std::fmt;

#[derive(Debug, Default)]
pub struct Turn {
    pub master_index: Option<usize>,
    cards: Deck,
}

impl Turn {
    pub fn take_cards_except_fool(self) -> Deck {
        Deck::new(
            self.cards
                .into_iter()
                .filter(|card| card.is_fool())
                .collect(),
        )
    }
    pub fn put(&mut self, card: Card) {
        self.cards.push(card);
    }
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    pub fn called(&self) -> Option<&Card> {
        for c in self.cards.iter() {
            if c.is_fool() {
                continue;
            } else {
                return Some(c);
            }
        }
        None
    }
    pub fn master_card(&self) -> Option<&Card> {
        self.master_index.map(|index| &self.cards[index])
    }
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Turn cards: \n{}", &self.cards.repr())?;
        if let Some(called) = self.called() {
            write!(f, "\nCalled: {}", &called.symbol())?;
        }
        if let Some(master) = self.master_card() {
            write!(f, "\nMaster: {}", &master)?;
        }
        Ok(())
    }
}
