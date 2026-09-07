use crate::card::Card;
use crate::deck::Deck;
use std::fmt;

#[derive(Clone, Debug, Default)]
pub struct Turn {
    pub master_index: Option<usize>,
    cards: Deck,
}

impl Turn {
    /// Cards effectively won by the master of this turn.
    ///
    /// RULE: when the fool stays with its owner, it does not go to the master of the turn.
    #[must_use]
    pub fn take_cards(self, fool_kept_by_owner: bool) -> Deck {
        if fool_kept_by_owner {
            Deck::new(
                self.cards
                    .into_iter()
                    .filter(|card| !card.is_fool())
                    .collect(),
            )
        } else {
            self.cards
        }
    }
    pub fn put(&mut self, card: Card) {
        self.cards.push(card);
    }
    #[must_use]
    pub const fn len(&self) -> usize {
        self.cards.len()
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    #[must_use]
    pub fn called(&self) -> Option<&Card> {
        // RULE: when the fool is played first, the called color is given by the next card
        self.cards.iter().find(|card| !card.is_fool())
    }
    #[must_use]
    pub const fn cards(&self) -> &Deck {
        &self.cards
    }
    #[must_use]
    pub fn master_card(&self) -> Option<&Card> {
        self.master_index.map(|index| self.cards.get(index))?
    }
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.cards)
    }
}

#[cfg(test)]
mod tests {
    use super::Turn;
    use crate::card::Card;
    use crate::suit::Suit;
    use crate::suit_value::SuitValue;
    use crate::trump::Trump;

    fn turn_with_fool() -> Turn {
        let mut turn = Turn::default();
        turn.put(Card::normal(Suit::Heart, SuitValue::King));
        turn.put(Card::Trump(Trump::Fool));
        turn.put(Card::normal(Suit::Heart, SuitValue::_3));
        turn.put(Card::Trump(Trump::_21));
        turn
    }

    #[test]
    fn master_wins_everything_but_a_kept_fool() {
        let won = turn_with_fool().take_cards(true);
        assert_eq!(won.len(), 3);
        assert!(!won.has_fool());
    }

    #[test]
    fn master_wins_a_lost_fool_too() {
        let won = turn_with_fool().take_cards(false);
        assert_eq!(won.len(), 4);
        assert!(won.has_fool());
    }

    #[test]
    fn called_color_skips_the_fool() {
        let mut turn = Turn::default();
        turn.put(Card::Trump(Trump::Fool));
        turn.put(Card::normal(Suit::Club, SuitValue::_5));
        assert_eq!(turn.called(), Some(&Card::normal(Suit::Club, SuitValue::_5)));
    }
}
