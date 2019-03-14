use std::fmt;
use crate::deck::Deck;
use crate::card::Card;

#[derive(Debug, Default)]
pub struct Turn {
    pub master_index: Option<usize>,
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
        if let Some(index) = self.master_index {
            Some(&self.cards.0[index])
        } else {
            None
        }
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

#[test]
fn turn_tests() {
}
