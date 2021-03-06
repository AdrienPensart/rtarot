use std::fmt;
use crate::deck::Deck;
use crate::card::Card;

#[derive(Debug, Default)]
pub struct Turn {
    pub master_index: Option<usize>,
    pub fool_played: bool,
    cards: Deck,
}

impl Turn {
    pub fn take(&mut self) -> Deck {
        Deck(self.cards.0.drain(..).collect())
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
        for c in &self.cards.0 {
            if c.is_fool() {
                continue
            } else {
                return Some(c)
            }
        }
        None
    }
    pub fn master_card(&self) -> Option<&Card> {
        self.master_index.map(|index| &self.cards.0[index])
    }
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Turn cards: {}", &self.cards)?;
        if let Some(called) = self.called() {
            write!(f, "\nCalled: {}", &called)?;
        }
        if let Some(master) = self.master_card() {
            write!(f, "\nMaster: {}", &master)?;
        }
        Ok(())
    }
}
