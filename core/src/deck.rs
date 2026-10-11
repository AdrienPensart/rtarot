use derive_more::{Deref, Index, IntoIterator};
use derive_new::new;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::Rng;
use rand::seq::SliceRandom;
use std::fmt;
use strum::IntoEnumIterator;

use crate::card::Card;
use crate::constants::{MAX_CARDS, MIN_CUT};
use crate::errors::TarotErrorKind;
use crate::points::Points;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::Discardable;
use crate::trump::Trump;

#[derive(new, Default, PartialEq, Eq, Clone, Debug, IntoIterator, Index, Deref)]
#[deref(forward)]
pub struct Deck(Vec<Card>);

impl Deck {
    /// The 78 cards, in a fixed order.
    #[must_use]
    pub fn full() -> Self {
        Self(
            Trump::iter()
                .map(Card::Trump)
                .chain(
                    Suit::iter()
                        .cartesian_product(SuitValue::iter())
                        .map(|(c, cv)| Card::normal(c, cv)),
                )
                .collect(),
        )
    }
    /// The 78 cards, shuffled with the caller's generator.
    pub fn shuffled<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let mut deck = Self::full();
        deck.0.shuffle(rng);
        deck
    }
    /// Lift the top `depth` cards and put them under the rest.
    ///
    /// RULE: a cut leaves at least three cards on each side.
    pub fn cut(&mut self, depth: usize) -> Result<(), TarotErrorKind> {
        if depth < MIN_CUT || depth + MIN_CUT > self.len() {
            return Err(TarotErrorKind::InvalidCut(depth));
        }
        self.0.rotate_left(depth);
        Ok(())
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
    pub const fn is_chelem(&self) -> bool {
        // RULE: a deck is a chelem when it holds every card of the game
        self.len() == MAX_CARDS
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
    pub const fn len(&self) -> usize {
        self.0.len()
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[must_use]
    pub fn petit_sec(&self) -> bool {
        // RULE: the petit is sec when it is the only trump,
        // holding the excuse is enough to avoid it
        !self.has_fool() && self.has_petit() && self.count_trumps() == 1
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
            .filter(|card| card.is_oudler())
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
        // RULE: a misere of heads is a hand without any figure card
        !self.iter().any(|card| match card {
            Card::Normal(n) => n.points() > OrderedFloat(0.5),
            Card::Trump(_) => false,
        })
    }
    #[must_use]
    pub fn give(&mut self, size: usize) -> Self {
        Self(self.0.drain(0..size).collect())
    }
    #[must_use]
    pub fn give_all(&mut self) -> Self {
        Self(std::mem::take(&mut self.0))
    }
    pub fn give_fool(&mut self) -> Option<Card> {
        let index = self.iter().position(|card| card.is_fool())?;
        Some(self.remove(index))
    }
    pub fn give_low(&mut self) -> Option<Card> {
        // RULE: a low card is worth half a point, a color card is given rather than a trump
        let low = |card: &Card| card.points() == OrderedFloat(0.5);
        let index = self
            .iter()
            .position(|card| low(card) && !card.is_trump())
            .or_else(|| self.iter().position(low))?;
        Some(self.remove(index))
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
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.iter().join(" "))
    }
}

impl Points for Deck {
    fn points(&self) -> OrderedFloat<f64> {
        // Cards are worth what they are worth: the compensation owed for a kept fool
        // is modelled by an explicit card exchange, not by a special case here.
        let mut total = OrderedFloat(0.0);
        for card in self.iter() {
            total += card.points();
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::Deck;
    use crate::card::Card;
    use crate::constants::{MAX_CARDS, MAX_POINTS};
    use crate::points::Points;
    use crate::suit::Suit;
    use crate::suit_value::SuitValue;
    use crate::traits::Discardable;
    use crate::trump::Trump;
    use ordered_float::OrderedFloat;

    #[test]
    fn a_full_deck_is_a_chelem_worth_all_the_points() {
        let deck = Deck::full();
        assert_eq!(deck.len(), MAX_CARDS);
        assert_eq!(deck.points(), MAX_POINTS);
        assert!(deck.is_chelem());
    }

    #[test]
    fn a_cut_puts_the_top_under_without_losing_a_card() {
        let full = Deck::full();
        let mut cut = full.clone();
        cut.cut(10).expect("ten cards is a fair cut");
        assert_eq!(cut.len(), MAX_CARDS);
        assert_eq!(cut.first(), full.get(10));
        assert_eq!(cut.get(MAX_CARDS - 10), full.first());
        assert!(cut.is_chelem());
    }

    #[test]
    fn a_cut_must_leave_three_cards_on_each_side() {
        let mut deck = Deck::full();
        assert!(deck.cut(2).is_err());
        assert!(deck.cut(MAX_CARDS - 2).is_err());
        assert!(deck.cut(3).is_ok());
        assert!(deck.cut(MAX_CARDS - 3).is_ok());
    }

    #[test]
    fn an_incomplete_deck_is_not_a_chelem() {
        let mut deck = Deck::full();
        let _removed = deck.remove(0);
        assert!(!deck.is_chelem());
    }

    #[test]
    fn a_lone_fool_is_worth_its_oudler_points() {
        let fool = Deck::new(vec![Card::Trump(Trump::Fool)]);
        assert!(fool.only_fool());
        assert_eq!(fool.points(), OrderedFloat(4.5));
    }

    #[test]
    fn misere_of_heads_needs_no_figure_card() {
        let low = Deck::new(vec![
            Card::normal(Suit::Heart, SuitValue::_3),
            Card::normal(Suit::Spade, SuitValue::_5),
            Card::Trump(Trump::_21),
        ]);
        let with_a_head = Deck::new(vec![
            Card::normal(Suit::Heart, SuitValue::_3),
            Card::normal(Suit::Spade, SuitValue::Queen),
        ]);
        assert!(low.misere_tete());
        assert!(!with_a_head.misere_tete());
    }

    #[test]
    fn petit_sec_is_the_petit_as_only_trump() {
        let petit_alone = Deck::new(vec![
            Card::Trump(Trump::Petit),
            Card::normal(Suit::Heart, SuitValue::King),
        ]);
        let another_trump = Deck::new(vec![
            Card::Trump(Trump::_2),
            Card::normal(Suit::Heart, SuitValue::King),
        ]);
        let no_trump_at_all = Deck::new(vec![Card::normal(Suit::Heart, SuitValue::King)]);
        assert!(petit_alone.petit_sec());
        assert!(!another_trump.petit_sec());
        assert!(!no_trump_at_all.petit_sec());
    }

    #[test]
    fn the_excuse_saves_the_petit_from_being_sec() {
        let petit_and_excuse = Deck::new(vec![
            Card::Trump(Trump::Fool),
            Card::Trump(Trump::Petit),
            Card::normal(Suit::Heart, SuitValue::King),
        ]);
        assert!(!petit_and_excuse.petit_sec());
    }

    #[test]
    fn oudlers_are_counted_and_scored() {
        let deck = Deck::new(vec![
            Card::Trump(Trump::Fool),
            Card::Trump(Trump::Petit),
            Card::Trump(Trump::_10),
            Card::normal(Suit::Club, SuitValue::King),
        ]);
        assert_eq!(deck.count_oudlers(), 2);
        assert_eq!(deck.count_trumps(), 3);
        assert_eq!(deck.points_for_oudlers(), Ok(OrderedFloat(41.0)));
    }

    #[test]
    fn a_color_card_is_given_before_a_trump() {
        let mut deck = Deck::new(vec![
            Card::Trump(Trump::_10),
            Card::normal(Suit::Club, SuitValue::_3),
        ]);
        assert_eq!(
            deck.give_low(),
            Some(Card::normal(Suit::Club, SuitValue::_3))
        );
    }

    #[test]
    fn the_fool_is_given_only_when_asked_for() {
        let mut deck = Deck::new(vec![Card::Trump(Trump::Fool)]);
        assert_eq!(deck.give_low(), None);
        assert_eq!(deck.give_fool(), Some(Card::Trump(Trump::Fool)));
        assert!(deck.is_empty());
    }

    #[test]
    fn kings_and_oudlers_are_never_discardable() {
        assert!(!Card::normal(Suit::Club, SuitValue::King).discardable());
        assert!(!Card::normal(Suit::Club, SuitValue::King).discardable_forced());
        assert!(!Card::Trump(Trump::_10).discardable());
        assert!(Card::Trump(Trump::_10).discardable_forced());
        assert!(!Card::Trump(Trump::Petit).discardable_forced());
    }
}
