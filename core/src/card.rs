use crate::normal::Normal;
use crate::points::Points;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::{Color, Discardable, Power, Representation};
use crate::trump::Trump;
use ordered_float::OrderedFloat;
use std::fmt;

#[derive(Copy, Ord, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Card {
    Trump(Trump),
    Normal(Normal),
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Trump(t) => write!(f, "{t}"),
            Self::Normal(n) => write!(f, "{n}"),
        }
    }
}

impl Points for Card {
    fn points(&self) -> OrderedFloat<f64> {
        match self {
            Self::Trump(v) => v.points(),
            Self::Normal(n) => n.points(),
        }
    }
}

impl Power for Card {
    fn power(&self) -> usize {
        match self {
            Self::Trump(v) => *v as usize + SuitValue::King as usize,
            Self::Normal(n) => n.power(),
        }
    }
}

impl Discardable for Card {
    fn discardable(&self) -> bool {
        match self {
            Self::Trump(t) => t.discardable(),
            Self::Normal(n) => n.discardable(),
        }
    }
    fn discardable_forced(&self) -> bool {
        match self {
            Self::Trump(t) => t.discardable_forced(),
            Self::Normal(n) => n.discardable_forced(),
        }
    }
}

impl Card {
    #[must_use]
    pub const fn normal(suit: Suit, value: SuitValue) -> Self {
        Self::Normal(Normal::new_const(suit, value))
    }
    #[must_use]
    pub const fn is_fool(self) -> bool {
        matches!(self, Self::Trump(Trump::Fool))
    }
    #[must_use]
    pub const fn is_trump(self) -> bool {
        matches!(self, Self::Trump(_))
    }
    #[must_use]
    pub const fn is_oudler(self) -> bool {
        match self {
            Self::Trump(c) => c.is_oudler(),
            Self::Normal(_) => false,
        }
    }
    #[must_use]
    pub fn master(self, arg: Self) -> bool {
        match (&self, &arg) {
            (Self::Trump(c), Self::Normal(_)) => c != &Trump::Fool,
            (Self::Normal(_), Self::Trump(c)) => c == &Trump::Fool,
            (Self::Normal(n1), Self::Normal(n2)) => {
                n1.suit() != n2.suit() || n1.value() > n2.value()
            }
            (Self::Trump(v1), Self::Trump(v2)) => v1 > v2,
        }
    }
}

impl Representation for Card {
    fn symbol(&self) -> &'static str {
        match self {
            Self::Normal(n) => n.symbol(),
            Self::Trump(t) => t.symbol(),
        }
    }
    fn color(&self) -> Color {
        match self {
            Self::Normal(n) => n.color(),
            Self::Trump(t) => t.color(),
        }
    }
    fn art(&self) -> &'static str {
        match self {
            Self::Normal(n) => n.art(),
            Self::Trump(t) => t.art(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Card;
    use crate::points::Points;
    use crate::suit::Suit;
    use crate::suit_value::SuitValue;
    use crate::traits::Discardable;
    use crate::trump::Trump;
    use ordered_float::OrderedFloat;

    #[test]
    fn a_trump_beats_a_color_but_the_fool_never_does() {
        let petit = Card::Trump(Trump::Petit);
        let fool = Card::Trump(Trump::Fool);
        let spade_1 = Card::normal(Suit::Spade, SuitValue::_1);
        assert!(petit.master(spade_1));
        assert!(!spade_1.master(petit));
        assert!(!fool.master(spade_1));
        assert!(spade_1.master(fool));
    }

    #[test]
    fn the_highest_card_of_the_asked_color_stays_master() {
        let spade_3 = Card::normal(Suit::Spade, SuitValue::_3);
        let spade_10 = Card::normal(Suit::Spade, SuitValue::_10);
        let diamond_3 = Card::normal(Suit::Diamond, SuitValue::_3);
        assert!(!spade_3.master(spade_10));
        assert!(spade_10.master(spade_3));
        // a card of another color never takes the lead from the asked one
        assert!(spade_10.master(diamond_3));
    }

    #[test]
    fn the_highest_trump_stays_master() {
        assert!(Card::Trump(Trump::_21).master(Card::Trump(Trump::Petit)));
        assert!(!Card::Trump(Trump::Petit).master(Card::Trump(Trump::_2)));
    }

    #[test]
    fn oudlers_and_kings_are_worth_four_and_a_half() {
        assert_eq!(Card::Trump(Trump::_21).points(), OrderedFloat(4.5));
        assert_eq!(Card::Trump(Trump::Fool).points(), OrderedFloat(4.5));
        assert_eq!(
            Card::normal(Suit::Club, SuitValue::King).points(),
            OrderedFloat(4.5)
        );
        assert_eq!(Card::Trump(Trump::_2).points(), OrderedFloat(0.5));
    }

    #[test]
    fn oudlers_and_kings_are_never_discardable() {
        for card in [
            Card::Trump(Trump::Petit),
            Card::Trump(Trump::Fool),
            Card::Trump(Trump::_21),
            Card::normal(Suit::Club, SuitValue::King),
        ] {
            assert!(!card.discardable());
            assert!(!card.discardable_forced());
        }
        assert!(!Card::Trump(Trump::_2).discardable());
        assert!(Card::Trump(Trump::_2).discardable_forced());
        assert!(Card::normal(Suit::Club, SuitValue::Queen).discardable());
    }

    #[test]
    fn a_card_displays_as_its_value_and_suit() {
        assert_eq!(
            Card::normal(Suit::Heart, SuitValue::King).to_string(),
            "K\u{2665}"
        );
        assert_eq!(Card::Trump(Trump::_21).to_string(), "#21");
        assert_eq!(Card::Trump(Trump::Fool).to_string(), "\u{1F0CF}");
    }
}
