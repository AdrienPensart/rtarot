use std::fmt;
use crate::traits::*;
use crate::color::*;
use crate::trump::*;

#[derive(Copy, Ord, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Card {
    Trump(TrumpValue),
    Color(Color, ColorValue)
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Card::Trump(t) => write!(f, "{}", t),
            Card::Color(c, v) => write!(f, "{} : {}", c, v)
        }
    }
}

impl Points for Card {
    fn points(&self) -> f64 {
        match self {
            Card::Trump(c) => c.points(),
            Card::Color(_, v) => v.points()
        }
    }
}

impl Discardable for Card {
    fn discardable(&self) -> bool {
        match self {
            Card::Trump(t) => t.discardable(),
            Card::Color(_, v) => v.discardable()
        }
    }
    fn discardable_forced(&self) -> bool {
        match self {
            Card::Trump(t) => t.discardable_forced(),
            Card::Color(_, v) => v.discardable_forced()
        }
    }
}

impl Card {
    pub fn is_fool(self) -> bool {
        match self {
            Card::Trump(v) => v == TrumpValue::Fool,
            _ => false
        }
    }
    pub fn is_trump(self) -> bool {
        match self {
            Card::Trump(_) => true,
            _ => false
        }
    }
    pub fn is_oudler(self) -> bool {
        match self {
            Card::Trump(c) => c.is_oudler(),
            _ => false
        }
    }
    pub fn master(self, arg: Card) -> bool {
        match (&self, &arg) {
            (Card::Trump(c), Card::Color(_, _)) => c != &TrumpValue::Fool,
            (Card::Color(_, _), Card::Trump(c)) => c == &TrumpValue::Fool,
            (Card::Color(c1, v1), Card::Color(c2, v2)) => c1 != c2 || v1 > v2,
            (Card::Trump(v1), Card::Trump(v2)) => v1 > v2,
        }
    }
}

#[test]
fn card_tests() {
    use std::f64::EPSILON;
    let trump_2 = Card::Trump(TrumpValue::_2);
    let petit = Card::Trump(TrumpValue::Petit);
    let fool = Card::Trump(TrumpValue::Fool);
    let _21 = Card::Trump(TrumpValue::_21);
    let spade_1 = Card::Color(Color::Spade, ColorValue::_1);
    let spade_2 = Card::Color(Color::Spade, ColorValue::_2);
    let spade_3 = Card::Color(Color::Spade, ColorValue::_3);
    let spade_10 = Card::Color(Color::Spade, ColorValue::_10);
    let diamond_3 = Card::Color(Color::Diamond, ColorValue::_3);

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
    assert!(!_21.discardable());
    assert!(!petit.discardable_forced());
    assert!(!fool.discardable_forced());
    assert!(!_21.discardable_forced());
    assert!(_21.points() - 4.5 < EPSILON);
}

