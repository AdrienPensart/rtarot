use crate::normal::Normal;
use crate::points::HasPoints;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::{Discardable, Power, Representation};
use crate::trump::Trump;
use colored::ColoredString;
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

impl HasPoints for Card {
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
    pub fn normal(suit: Suit, value: SuitValue) -> Self {
        Self::Normal(Normal::new(suit, value))
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
    fn colored_symbol(&self) -> ColoredString {
        match self {
            Self::Normal(n) => n.colored_symbol(),
            Self::Trump(t) => t.colored_symbol(),
        }
    }
    fn color(&self) -> &'static str {
        match self {
            Self::Normal(n) => n.color(),
            Self::Trump(t) => t.color(),
        }
    }
    fn repr(&self) -> ColoredString {
        match self {
            Self::Normal(n) => n.repr(),
            Self::Trump(t) => t.repr(),
        }
    }
    fn full_repr(&self) -> ColoredString {
        match self {
            Self::Normal(n) => n.full_repr(),
            Self::Trump(t) => t.full_repr(),
        }
    }
}

#[test]
fn card_tests() {
    let trump_2 = Card::Trump(Trump::_2);
    println!("{}", trump_2.repr());
    let petit = Card::Trump(Trump::Petit);
    println!("{}", petit.repr());
    let fool = Card::Trump(Trump::Fool);
    println!("{}", fool.repr());
    let unassailable = Card::Trump(Trump::_21);
    let spade_1 = Card::normal(Suit::Spade, SuitValue::_1);
    let spade_2 = Card::normal(Suit::Spade, SuitValue::_2);
    let spade_3 = Card::normal(Suit::Spade, SuitValue::_3);
    let spade_10 = Card::normal(Suit::Spade, SuitValue::_10);
    println!("{}", spade_10.repr());
    let diamond_3 = Card::normal(Suit::Diamond, SuitValue::_3);
    println!("{}", diamond_3.repr());
    let heart_4 = Card::normal(Suit::Heart, SuitValue::_4);
    println!("{}", heart_4.repr());
    let club_king = Card::normal(Suit::Club, SuitValue::King);
    println!("{}", club_king.repr());

    assert!(!spade_3.master(spade_10));
    assert!(!petit.master(trump_2));
    assert!(petit.master(spade_1));
    assert!(!spade_1.master(petit));
    assert!(spade_2.master(spade_1));
    assert!(diamond_3.master(spade_2));
    assert!(diamond_3.master(fool));
    assert!(!fool.master(spade_1));

    assert!(!petit.discardable());
    assert!(!fool.discardable());
    assert!(!unassailable.discardable());
    assert!(!petit.discardable_forced());
    assert!(!fool.discardable_forced());
    assert!(!unassailable.discardable_forced());
    assert_eq!(unassailable.points(), 4.5);
}
